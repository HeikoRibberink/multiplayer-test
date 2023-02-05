#![cfg(test)]
use assert_in_order::*;
use bevy::log::{self, Level, LogPlugin};
use bevy::{app::AppExit, prelude::*};
use multiplayer_test::client::FromServer;
use multiplayer_test::connection::ext::Event;
use multiplayer_test::server::{FromClient, Server, ServerPlugin};
use multiplayer_test::{
	self,
	client::{Client, ClientPlugin},
	connection::ConnectionError,
	MultiplayerPlugin, RuntimeResource,
};

in_order_init!(TEST);

#[test]
fn many_messages() -> Result<(), Box<dyn std::error::Error>> {
	let rt = tokio::runtime::Builder::new_multi_thread()
		// .worker_threads(1)
		.enable_io()
		.build()?;

	App::new()
		.add_plugins(MinimalPlugins)
		.add_plugin(LogPlugin {
			level: Level::WARN,
			..default()
		})
		.add_plugin(MultiplayerPlugin)
		.add_plugin(ClientPlugin::<String, String>::default())
		.add_plugin(ServerPlugin::<String, String>::default())
		.insert_resource(RuntimeResource(rt))
		.add_startup_system(setup)
		.add_system(client_on_error)
		.add_system(server_on_connect)
		.add_system(client_on_connect)
		.add_system(server_on_msg)
		.run();
	Ok(())
}

pub fn setup(
	mut client: ResMut<Client<String, String>>,
	mut server: ResMut<Server<String, String>>,
	rt: Res<RuntimeResource>,
) {
	let addr = "127.0.0.1:8080";
	info!("Binding to {}", addr);
	in_order!(TEST: binding);
	server.bind(addr, rt.handle().clone());
	info!("Connecting to {}", addr);
	in_order!(TEST: connecting after binding);
	client.connect(addr, rt.handle().clone());
}

pub fn server_on_connect(
	mut connecteds: EventReader<FromClient<String>>,
) {
	for event in connecteds.iter() {
		let Event::Connected(addr, id) = &**event else {
			continue
		};
		info!("Client {} connected: {:?}", id, &addr);
		// server.connections.remove(&id);
	}
}

pub fn server_on_msg(mut events: EventReader<FromClient<String>>, mut i: Local<u32>, mut exit: EventWriter<AppExit>) {
	for event in events.iter() {
		let Event::Message(msg, id) = &**event else {
			continue;
		};
		log::debug!("Msg received from client {}: {:?}", id, msg);
		// in_order!(TEST: receiving after sending);
		*i += 1;
		if *i >= 100 {
			exit.send(AppExit);
		}
	}
}

pub fn client_on_connect(
	mut connecteds: EventReader<FromServer<String>>,
	client: Res<Client<String, String>>,
) {
	for event in connecteds.iter() {
		let Event::Connected(addr, id) = &**event else {
			continue;
		};
		info!("Client {} connected to server: {:?}", id, &addr);
		in_order!(TEST: connected after connecting);
		let client = (&*client).as_ref().unwrap();
		in_order!(TEST: sending after connected);
		for i in 0..100 {
			client.send_blocking(format!("{i}")).unwrap(); //TODO: Messages are send, but never received...
		}
	}
}

pub fn client_on_error(
	mut events: EventReader<FromServer<String>>,
) {
	for ev in events.iter() {
		let Event::Error(err, id) = &**ev else {
			continue
		};
		match err {
			ConnectionError::Disconnected => {
				panic!("Client {} got disconnected.", id);
			}
			_ => {
				dbg!(err);
			}
		}
	}
}
