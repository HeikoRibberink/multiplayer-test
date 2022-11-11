use std::net::SocketAddr;
use std::sync::Arc;

use async_channel::{unbounded, Receiver, SendError, Sender};
use bevy::prelude::*;

use futures::lock::Mutex;

use thiserror::Error;
use tokio::io::{self, split, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
// use tokio_rustls::rustls::{ClientConfig, OwnedTrustAnchor};
// use tokio_rustls::webpki::TrustAnchor;
// use tokio_rustls::{rustls, TlsConnector};
// use webpki_roots::TLS_SERVER_ROOTS;

use crate::messaging::{self, recv_msg, ToClientMsg, ToServerMsg, ReflectToClientMsg};


//TODO: maybe refactor client and server to use similar messaging code
//TODO: automatically turn a FromClient::Received msg into typed event, that can be registered with an AppBuilder
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
	fn build(&self, app: &mut App) {
		app.add_system_to_stage(CoreStage::First, Client::event_system);

		//Create channels
		let (send_to_internal, recv_at_internal) = unbounded::<ToClient>();
		let (send_at_internal, recv_from_internal) = unbounded::<FromClient>();

		let internal_client = InternalClient {
			control_recv: recv_at_internal,
			event_send: send_at_internal,
			connection: None,
		};

		let rt = app.world.resource::<Runtime>();
		let task = rt.spawn(internal_client.run());

		let client = Client {
			send: send_to_internal,
			recv: recv_from_internal,
			_task: task,
		};

		app.add_event::<FromClient>();
		app.insert_resource(client);
	}
}

//Public API
pub struct Client {
	send: Sender<ToClient>,
	recv: Receiver<FromClient>,
	_task: JoinHandle<Result<(), ClientError>>,
}

impl Client {
	pub fn connect(&mut self, addr: SocketAddr) -> Result<(), SendError<SocketAddr>> {
		self.send
			.send_blocking(ToClient::Connect(addr))
			.map_err(|err| {
				SendError({
					if let ToClient::Connect(addr) = err.0 {
						addr
					} else {
						panic!("WTF?! This should not be able to happen...")
					}
				})
			})
	}

	pub fn disconnect(&mut self) -> Result<(), SendError<()>> {
		self.send
			.send_blocking(ToClient::Disconnect)
			.map_err(|_| SendError(()))
	}

	pub fn send_msg(
		&self,
		msg: Box<dyn ToServerMsg>,
	) -> Result<(), SendError<Box<dyn ToServerMsg>>> {
		self.send.send_blocking(ToClient::Send(msg)).map_err(|err| {
			SendError({
				if let ToClient::Send(msg) = err.0 {
					msg
				} else {
					panic!("WTF?! This should not be able to happen...")
				}
			})
		})
	}

	fn event_system(client: Res<Client>, mut e_writer: EventWriter<FromClient>) {
		while let Ok(from) = client.recv.try_recv() {
			e_writer.send(from);
		}
	}
}

#[derive(Debug)]
enum ToClient {
	Connect(SocketAddr),
	Disconnect,
	Send(Box<dyn ToServerMsg>),
}

#[derive(Debug)]
pub enum FromClient {
	Connected,
	Disconnected,
	Received(Box<dyn ToClientMsg>),
}

//Internal API
struct InternalClient {
	control_recv: Receiver<ToClient>,
	event_send: Sender<FromClient>,
	connection: Option<Connection>,
}

struct Connection {
	read: Arc<Mutex<Option<BufReader<ReadHalf<TcpStream>>>>>,
	write: BufWriter<WriteHalf<TcpStream>>,
	receive_task: JoinHandle<Result<(), ClientError>>,
}

#[derive(Error, Debug)]
pub enum ClientError {
	#[error("IO Error.")]
	IOError(#[from] io::Error),
	#[error("The client hasn't been connected (yet).")]
	Disconnected,
	#[error("Unable to serialize message.")]
	SerializationError(#[from] postcard::Error),
}

impl From<messaging::SendError> for ClientError {
	fn from(o: messaging::SendError) -> Self {
		match o {
			messaging::SendError::IOError(err) => Self::IOError(err),
			messaging::SendError::SerializationError(err) => Self::SerializationError(err),
		}
	}
}

impl From<messaging::RecvError> for ClientError {
	fn from(o: messaging::RecvError) -> Self {
		match o {
			messaging::RecvError::IOError(err) => Self::IOError(err),
			messaging::RecvError::DeserializationError(err) => Self::SerializationError(err),
		}
	}
}

impl InternalClient {
	async fn run(mut self) -> Result<(), ClientError> {
		while let Ok(recv) = self.control_recv.recv().await {
			use ToClient::*;
			match recv {
				Connect(addr) => self.connect(addr).await?,
				Disconnect => self.disconnect().await?,
				Send(msg) => self.send(msg).await?,
			}
		}
		Ok(())
	}

	async fn listen(
		read_arc: Arc<Mutex<Option<BufReader<ReadHalf<TcpStream>>>>>,
		event_send: Sender<FromClient>,
	) -> Result<(), ClientError> {
		let mut read_guard = read_arc
			.try_lock()
			.expect("No other task should be able to have the lock of this reader.");
		let read = read_guard.as_mut().expect(
			"The mutex should contain a reader as listen() is only started after a connection.",
		);

		loop {
			let data = postcard::from_bytes(&*recv_msg(read).await?)?;
			event_send.send(FromClient::Received(data)).await.unwrap();
		}
	}

	async fn connect(&mut self, addr: SocketAddr) -> Result<(), ClientError> {
		//Init stream
		let stream = TcpStream::connect(addr).await?; //TODO: Panics with 'there is no reactor running, must be called from the context of a Tokio 1.x runtime'. After inspecting the backtrace, it seems
		println!("Finished connect!");
		let (read, write) = split(stream);

		let read = BufReader::new(read);
		let write = BufWriter::new(write);

		//Send read half to other task
		let read = Arc::new(Mutex::new(Some(read)));
		let receive_task = tokio::spawn(Self::listen(read.clone(), self.event_send.clone()));

		self.connection = Some(Connection {
			read,
			write,
			receive_task,
		});

		self.event_send.send(FromClient::Connected).await.unwrap();

		// let task = task_pool.spawn(self.listen());
		Ok(())
	}

	async fn disconnect(&mut self) -> Result<(), ClientError> {
		let connection = self.connection.take().ok_or(ClientError::Disconnected)?;
		let Connection {
			read,
			write,
			receive_task,
		} = connection;

		//Wait for the task to finish.
		receive_task.abort();
		receive_task.await; // Handle result

		let read = read
			.try_lock()
			.expect("Lock shouldn't be owned anymore because task has finished.")
			.take()
			.expect("Connection should have a BufReader.")
			.into_inner();

		let write = write.into_inner();

		let mut stream = read.unsplit(write);

		stream.shutdown().await?;

		self.event_send
			.send(FromClient::Disconnected)
			.await
			.unwrap();

		Ok(())
	}

	async fn send(&mut self, msg: Box<dyn ToServerMsg>) -> Result<(), ClientError> {
		let connection = self.connection.as_mut().ok_or(ClientError::Disconnected)?;
		messaging::send_msg(&mut connection.write, postcard::to_stdvec(&*msg)?).await?;
		Ok(())
	}
}
