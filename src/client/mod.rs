use bevy::prelude::*;
use tokio::net::ToSocketAddrs;

use tokio::runtime::Handle as RtHandle;

use crate::{
	messaging::NetMsg,
	NetStage, connection::{ConnectionHandle, ext::{ConnectionEvent, ConnErrorEvent, emit_messages_as_events}},
};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
	fn build(&self, app: &mut App) {
		app.add_system_to_stage(NetStage::Receive, Client::event_system)
		.insert_resource(Client { connection: None })
		.add_event::<ConnectionEvent<Box<dyn NetMsg>>>()
		.add_event::<ConnErrorEvent>()
		;
	}
}

pub struct Client {
	pub connection: Option<ConnectionHandle<Box<dyn NetMsg>, Box<dyn NetMsg>>>,
}

impl Client {
	pub fn connect<A: ToSocketAddrs + Send + Sync + 'static>(&mut self, rt: RtHandle, addr: A) {
		self.connection = Some(ConnectionHandle::connect(rt, addr));
	}
}

impl Client {
	pub fn event_system(
		client: Res<Client>,
		mut data_events: EventWriter<ConnectionEvent<Box<dyn NetMsg>>>,
		mut error_events: EventWriter<ConnErrorEvent>,
	) {
		let Some(ref conn) = client.connection else {
			return;
		};
		emit_messages_as_events(conn, &mut data_events, &mut error_events);
	}
}
