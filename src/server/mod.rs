use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use async_channel::{unbounded, Receiver, RecvError, SendError, Sender};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
	io,
	net::{TcpListener, ToSocketAddrs},
	runtime::Handle,
	task::{JoinError, JoinHandle},
};

use crate::connection::{
	ext::{self, ConnectedEvent},
	ConnectionHandle, ConnectionId,
};

mod plugin;

pub use plugin::*;

pub struct ServerHandle<S, R>
where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + Sync + 'static,
{
	pub connections: Arc<DashMap<ConnectionId, ConnectionHandle<S, R>>>,
	running: Arc<AtomicBool>,
	task: Option<JoinHandle<Result<(), ServerError>>>,
	from_task: Receiver<ConnectedEvent>,
	rt: Handle,
}

impl<S, R> ServerHandle<S, R>
where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + Sync + 'static,
{
	pub fn bind<A: ToSocketAddrs + Sync + Send + 'static>(rt: Handle, addr: A) -> Self {
		let connections = Arc::new(DashMap::new());
		let running = Arc::new(AtomicBool::new(true));

		let (to_handle, from_task) = unbounded::<ext::ConnectedEvent>();

		let server = InternalServer {
			connections: connections.clone(),
			running: running.clone(),
			rt: rt.clone(),
			to_handle,
		};

		let task = Some(rt.spawn(server.listen(addr)));

		Self {
			connections,
			running,
			task,
			from_task,
			rt,
		}
	}

	fn internal_disconnect_blocking(&mut self) -> Result<(), ServerError> {
		self.internal_disconnect()?;
		self.rt.block_on(async {
			//Cancellation safety: should be safe as we don't care about any data that hasn't been send or received after waiting.
			//Maybe use select! with a branch that waits for a specified time to not wait indefinitely
			self.task.take().unwrap().await
		})?
	}

	fn internal_disconnect(&self) -> Result<(), ServerError> {
		self.running.store(false, Ordering::Relaxed);
		if !self.from_task.close() {
			return Err(ServerError::Disconnected);
		};
		return Ok(());
	}

	pub fn disconnect_blocking(mut self) -> Result<(), ServerError> {
		self.internal_disconnect_blocking()
	}

	pub fn try_recv(&self) -> Result<Option<ConnectedEvent>, ServerError> {
		return match self.from_task.try_recv() {
			Ok(val) => Ok(Some(val)),

			Err(err) => match err {
				async_channel::TryRecvError::Empty => Ok(None),
				async_channel::TryRecvError::Closed => Err(ServerError::Disconnected),
			},
		};
	}

}

impl<S, R> Drop for ServerHandle<S, R>
where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + Sync + 'static,
{
	fn drop(&mut self) {
		if !self.running.load(Ordering::Relaxed) {
			self.internal_disconnect_blocking(); //TODO: how should the connection be ended?
		}
	}
}

struct InternalServer<S, R>
where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + Sync + 'static,
{
	connections: Arc<DashMap<ConnectionId, ConnectionHandle<S, R>>>,
	running: Arc<AtomicBool>,
	rt: Handle,
	to_handle: Sender<ConnectedEvent>,
}

impl<S, R> InternalServer<S, R>
where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + Sync + 'static,
{
	async fn listen<A: ToSocketAddrs + Sync + Sync + 'static>(
		self,
		addr: A,
	) -> Result<(), ServerError> {
		let listener = TcpListener::bind(addr).await.unwrap();

		while let Ok((stream, addr)) = listener.accept().await {
			let conn = ConnectionHandle::with_stream(self.rt.clone(), stream);
			if let Err(_err) = self.to_handle.send(ConnectedEvent(addr, conn.uuid)).await {
				// If the channel returns an error and running is true, unexpected disconnect.
				if self.running.load(Ordering::Relaxed) {
					return Err(ServerError::Disconnected);
				}
				// Otherwise, the handler has signaled a disconnect.
				break;
			}
			self.connections.insert(conn.uuid, conn);
		}

		Ok(())
	}
}

#[derive(Error, Debug)]
pub enum ServerError {
	#[error("IO Error.")]
	IOError(io::Error),
	#[error("Not connected, or unexpected disconnect.")]
	Disconnected,
	#[error("Unable to serialize message.")]
	SerializationError(#[from] postcard::Error),
	#[error("The underlying task returned an error on join.")]
	JoinError(#[from] JoinError),
}

impl From<RecvError> for ServerError {
	fn from(_v: RecvError) -> Self {
		Self::Disconnected
	}
}

impl<T> From<SendError<T>> for ServerError {
	fn from(_v: SendError<T>) -> Self {
		Self::Disconnected
	}
}

impl From<io::Error> for ServerError {
	fn from(err: io::Error) -> Self {
		match err.kind() {
			//TODO: add other kinds of io::Error that may be converted into ServerError
			io::ErrorKind::UnexpectedEof => return Self::Disconnected,
			_ => {
				return Self::IOError(err);
			}
		}
	}
}
