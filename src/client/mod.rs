use bevy::prelude::*;
use bevy::reflect::FromType;
use bevy::{
	utils::Uuid,
};
use hashbrown::HashMap;

use crate::serialization::{ToServerMsg, ReflectToServerMsg};
// use crate::serialization::components::{ReflectNetComponent, NetComponent};
use crate::serialization::{bundle::NetBundle, NetEntity};
use crossbeam_channel::Receiver;
use postcard;
use serde::{de::DeserializeSeed, Deserialize, Serialize, Serializer};
use tokio_rustls::client::TlsStream;

//Probably use the tokio runtime for IO (with tokio-rustls), send bevy events to that server, and receive events from the server.

pub struct Client {
	entity_registry: HashMap<Uuid, NetEntity>,
	//TODO: stream: TlsStream<Msg<>>,
	// msg_recv: Receiver<Box<dyn ServerMsg>>,
}

impl Client {
	pub fn sender_system(mut event_reader: EventReader<Box<dyn ToServerMsg>>, client: Res<Client>) {
		for event in event_reader.iter() {
			let event = (&**event);
			let ser = postcard::to_stdvec(event);

		}
	}
}

//Maybe spawn entities with a custom [command](https://docs.rs/bevy/latest/bevy/prelude/struct.Commands.html#method.add)