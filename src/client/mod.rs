use bevy::utils::Uuid;
use hashbrown::HashMap;

use crate::networking::{NetworkedEntity, Msg};
use tokio_rustls::client::TlsStream;
use crossbeam_channel::Receiver;

//Probably use the tokio runtime for IO (with tokio-rustls), send bevy events to that server, and receive events from the server.

pub struct Client {
	entity_registry: HashMap<Uuid, NetworkedEntity>,
	//TODO: stream: TlsStream<Msg<>>,	
	//TODO: msg_recv: Receiver<>,
}