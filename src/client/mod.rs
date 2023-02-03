use bevy::prelude::*;
use tokio::net::ToSocketAddrs;

use tokio::runtime::Handle;

use crate::{
	connection::{
		ext::{emit_messages_as_events, Event},
		ConnectionHandle,
	},
	messaging::NetMsg,
	NetStage,
};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
	fn build(&self, app: &mut App) {
		app.add_system_to_stage(NetStage::Receive, Client::event_system)
			.insert_resource(Client { connection: None })
			.add_event::<Event<NetMsg>>();
	}
}

#[derive(Resource)]
pub struct Client {
	pub connection: Option<ConnectionHandle<NetMsg, NetMsg>>,
	// pub connection: Option<ConnectionHandle<(), ()>>,
}

impl Client {
	pub fn connect<A: ToSocketAddrs + Send + Sync + 'static>(&mut self, rt: Handle, addr: A) {
		self.connection = Some(ConnectionHandle::connect(rt, addr));
	}
}

impl Client {
	pub fn event_system(
		client: Res<Client>,
		mut eventwriter: EventWriter<Event<NetMsg>>,
	) {
		let Some(ref conn) = client.connection else {
			return;
		};
		emit_messages_as_events(conn, &mut eventwriter);
	}
}
