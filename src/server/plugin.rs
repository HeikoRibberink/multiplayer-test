use std::net::SocketAddr;

use bevy::prelude::{EventWriter, Plugin, Res, Resource, DerefMut, Deref};

use crate::{
	connection::ext::Event,
	messaging::NetMsg,
	NetStage,
};

use super::ServerHandle;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_system_to_stage(NetStage::Receive, Server::event_system)
			.insert_resource(Server { server: None })
			.add_event::<FromClient<NetMsg>>();
	}
}

#[derive(Resource)]
pub struct Server {
	pub server: Option<ServerHandle<NetMsg, NetMsg>>,
}

#[derive(Deref, DerefMut)]
pub struct FromClient<R>(Event<R>);

impl<R> From<Event<R>> for FromClient<R> {
    fn from(value: Event<R>) -> Self {
        Self(value)
    }
}

impl Server {
	pub fn event_system(server: Res<Server>, mut eventwriter: EventWriter<FromClient<NetMsg>>) {
		let Some(ref server) = server.server else {
			return;
		};
		loop {
			let recv = server.try_recv();
			if let Ok(opt) = recv {
				let Some(val) = opt else {
					break;
				};
				eventwriter.send(Event::Connected(val.0, val.1).into());
			}
		}
		for conn in server.connections.iter() {
			loop {
				let recv = conn.try_recv();
				if let Ok(opt) = recv {
					let Some(val) = opt else {
						break;
					};
					eventwriter.send(Event::Message(val, conn.uuid).into());
				} else if let Err(err) = recv {
					eventwriter.send(Event::Error(err, conn.uuid).into());
					break;
				}
			}
		}
	}
}
