use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use async_channel::{unbounded, Receiver, RecvError, SendError, Sender};
use bevy::utils::Uuid;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
	io::{self, AsyncWriteExt, BufReader, BufWriter},
	net::{
		tcp::{OwnedReadHalf, OwnedWriteHalf},
		TcpStream, ToSocketAddrs,
	},
	runtime::Handle,
	task::{JoinError, JoinHandle},
};

use crate::messaging;

pub mod ext;

pub type ConnectionId = Uuid;

#[derive(Debug)]
pub struct ConnectionHandle<S, R>
where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + 'static,
{
	to_conn: Sender<S>,
	from_conn: Receiver<R>,
	pub uuid: ConnectionId,
	running: Arc<AtomicBool>,
	runtime: Handle,
	task: Option<JoinHandle<Result<(), ConnectionError>>>,
}

impl<S, R> ConnectionHandle<S, R>
where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + 'static,
{
	pub fn connect<A>(addr: A, rt: Handle) -> ConnectionHandle<S, R>
	where
		A: ToSocketAddrs + Send + 'static,
	{
		let (to_conn, from_handle) = unbounded::<S>();
		let (to_handle, from_conn) = unbounded::<R>();

		let running = Arc::new(AtomicBool::new(true));

		let connection = Connection {
			to_handle,
			from_handle,
			running: running.clone(),
		};

		let task = Some(rt.spawn(connection.connect(addr)));

		let uuid = Uuid::new_v4();

		Self {
			to_conn,
			from_conn,
			uuid,
			running,
			runtime: rt.clone(),
			task,
		}
	}

	pub fn with_stream(stream: TcpStream, rt: Handle) -> ConnectionHandle<S, R> {
		let (to_conn, from_handle) = unbounded::<S>();
		let (to_handle, from_conn) = unbounded::<R>();

		let running = Arc::new(AtomicBool::new(true));

		let connection = Connection {
			to_handle,
			from_handle,
			running: running.clone(),
		};

		let task = Some(rt.spawn(connection.run(stream)));

		let uuid = Uuid::new_v4();

		Self {
			to_conn,
			from_conn,
			uuid,
			running,
			runtime: rt.clone(),
			task,
		}
	}

	fn internal_disconnect_blocking(&mut self) -> Result<(), ConnectionError> {
		self.internal_disconnect()?;
		self.runtime.block_on(async {
			//Cancellation safety: should be safe as we don't care about any data that hasn't been send or received after waiting.
			//Maybe use select! with a branch that waits for a specified time to not wait indefinitely
			self.task.take().unwrap().await
		})?
	}

	fn internal_disconnect(&self) -> Result<(), ConnectionError> {
		self.running.store(false, Ordering::Relaxed);
		if !(self.to_conn.close() & self.from_conn.close()) {
			return Err(ConnectionError::Disconnected);
		};
		return Ok(());
	}

	pub fn disconnect_blocking(mut self) -> Result<(), ConnectionError> {
		self.internal_disconnect_blocking()
	}

	pub fn send_blocking(&self, data: S) -> Result<(), ConnectionError> {
		self.to_conn.send_blocking(data)?;
		Ok(())
	}

	pub fn is_running(&self) -> bool {
		self.running.load(Ordering::Relaxed)
	}

	pub fn try_recv(&self) -> Result<Option<R>, ConnectionError> {
		return match self.from_conn.try_recv() {
			Ok(val) => Ok(Some(val)),

			Err(err) => match err {
				async_channel::TryRecvError::Empty => Ok(None),
				async_channel::TryRecvError::Closed => Err(ConnectionError::Disconnected),
			},
		};
	}
}

impl<S, R> Drop for ConnectionHandle<S, R>
where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + 'static,
{
	fn drop(&mut self) {
		if !self.running.load(Ordering::Relaxed) {
			self.internal_disconnect_blocking(); //TODO: how should the connection be ended?
		}
	}
}

struct Connection<S, R>
where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + 'static,
{
	to_handle: Sender<R>,
	from_handle: Receiver<S>,
	running: Arc<AtomicBool>,
}

impl<S, R> Connection<S, R>
where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + 'static,
{
	async fn run(self, stream: TcpStream) -> Result<(), ConnectionError> {
		//Split the stream up to be able to split sending and receiving
		let (read, write) = stream.into_split();

		let mut read = BufReader::new(read);
		let mut write = BufWriter::new(write);

		//Spawn listening and write tasks.
		let output = tokio::select!(
			result = async {
				if let Err(err) = self.listen_to_stream(&mut read).await {
					self.stop()? /*?*/; //TODO: is this correct?
					Err(err)
				} else {
					Ok(())
				}
			} => result,
			result = async {
				if let Err(err) = self.write_to_stream(&mut write).await {
					self.stop()? /*?*/; //TODO: is this correct?
					Err(err)
				} else {
					Ok(())
				}
			} => result,
		);

		//Reunite halves
		let mut stream = read
			.into_inner()
			.reunite(write.into_inner())
			.expect("Both halves should be from the same TcpStream.");

		stream.shutdown().await?;

		//TODO: unexpected disconnects succesfully stop both threads and get here
		output
	}

	async fn connect<A: ToSocketAddrs>(self, addr: A) -> Result<(), ConnectionError> {
		let stream = TcpStream::connect(addr).await?;
		self.run(stream).await
	}

	async fn write_to_stream(
		&self,
		write: &mut BufWriter<OwnedWriteHalf>,
	) -> Result<(), ConnectionError> {
		while self.running.load(Ordering::Relaxed) {
			let Ok(msg) = self.from_handle.recv().await else {
				// If the channel returns an error and running is true, error.
				if self.running.load(Ordering::Relaxed) {
					return Err(ConnectionError::Disconnected)
				}
				// Otherwise, the handler has signaled a disconnect.
				break;
			};
			let bytes = postcard::to_stdvec(&msg)?;
			messaging::send_msg(write, bytes).await?;
		}

		Ok(())
	}

	async fn listen_to_stream(
		&self,
		read: &mut BufReader<OwnedReadHalf>,
	) -> Result<(), ConnectionError> {
		while self.running.load(Ordering::Relaxed) {
			match messaging::recv_msg(read).await {
				Ok(bytes) => {
					let data: R = postcard::from_bytes(&*bytes)?;
					if let Err(_err) = self.to_handle.send(data).await {
						// If the channel returns an error and running is true, error.
						if self.running.load(Ordering::Relaxed) {
							return Err(ConnectionError::Disconnected);
						}
						// Otherwise, the handler has signaled a disconnect.
						break;
					}
				}

				//TODO: The connection should be shut down when an error is returned.
				Err(error) => return Err(error.into()),
			}
		}

		Ok(())
	}

	fn stop(&self) -> Result<(), ConnectionError> {
		self.running.store(false, Ordering::Relaxed);
		if !(self.to_handle.close() & self.from_handle.close()) {
			return Err(ConnectionError::Disconnected);
		};
		return Ok(());
	}
}

#[derive(Error, Debug)]
pub enum ConnectionError {
	#[error("IO Error.")]
	IOError(io::Error),
	#[error("Not connected, or unexpected disconnect.")]
	Disconnected,
	#[error("Unable to serialize message.")]
	SerializationError(#[from] postcard::Error),
	#[error("The underlying task returned an error on join.")]
	JoinError(#[from] JoinError),
}

impl From<RecvError> for ConnectionError {
	fn from(_v: RecvError) -> Self {
		Self::Disconnected
	}
}

impl<T> From<SendError<T>> for ConnectionError {
	fn from(_v: SendError<T>) -> Self {
		Self::Disconnected
	}
}

impl From<io::Error> for ConnectionError {
	fn from(err: io::Error) -> Self {
		match err.kind() {
			//TODO: add other kinds of io::Error that may be converted into ConnectionError
			io::ErrorKind::UnexpectedEof => return ConnectionError::Disconnected,
			_ => {
				return Self::IOError(err);
			}
		}
	}
}
