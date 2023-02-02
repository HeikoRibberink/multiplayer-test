use std::net::SocketAddr;

use bevy::prelude::{EventWriter, Plugin, Res, Resource};

use crate::{
	connection::ext::{ErrorEvent, ConnectedEvent, MessageEvent},
	messaging::NetMsg,
	NetStage,
};

use super::ServerHandle;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_system_to_stage(NetStage::Receive, Server::event_system)
			.insert_resource(Server { server: None })
			.add_event::<ConnectedEvent>();
	}
}

#[derive(Resource)]
pub struct Server {
	pub server: Option<ServerHandle<NetMsg, NetMsg>>,
}

impl Server {
	pub fn event_system(server: Res<Server>, mut connecteds: EventWriter<ConnectedEvent>, mut connection_messages: EventWriter<MessageEvent<NetMsg>>, mut connection_errors: EventWriter<ErrorEvent>) {
		let Some(ref server) = server.server else {
			return;
		};
		loop {
			let recv = server.try_recv();
			if let Ok(opt) = recv {
				let Some(val) = opt else {
					break;
				};
				connecteds.send(val);
			}
		}
		for conn in server.connections.iter() {
			loop {
				let recv = conn.try_recv();
				if let Ok(opt) = recv {
					let Some(val) = opt else {
						break;
					};
					connection_messages.send(MessageEvent(val, conn.uuid));
				} else if let Err(err) = recv {
					connection_errors.send(ErrorEvent(err, conn.uuid));
					break;
				}
			}
		}
	}
}
