#![cfg(test)]
use bevy::log::{Level, LogPlugin};
use bevy::{app::AppExit, prelude::*};
use multiplayer_test::client::FromServer;
use multiplayer_test::connection::ext::Event;
use multiplayer_test::server::{Server, ServerPlugin, FromClient};
use multiplayer_test::{
	self,
	client::{Client, ClientPlugin},
	connection::{ConnectionError},
	MultiplayerPlugin, RuntimeResource,
};
use assert_in_order::*;

in_order_init!(TEST);

#[test]
fn connect_disconnect() -> Result<(), Box<dyn std::error::Error>> {
	let rt = tokio::runtime::Builder::new_multi_thread()
		.worker_threads(1)
		.enable_io()
		.build()?;

	App::new()
		.add_plugins(MinimalPlugins)
		.add_plugin(LogPlugin {
			level: Level::WARN,
			..default()
		})
		.add_plugin(MultiplayerPlugin)
		.add_plugin(ClientPlugin::<(), ()>::default())
		.add_plugin(ServerPlugin::<(), ()>::default())
		.insert_resource(RuntimeResource(rt))
		.add_startup_system(setup)
		.add_system(on_error)
		.add_system(on_connect)
		.run();
	Ok(())
}

pub fn setup(mut client: ResMut<Client<(), ()>>, mut server: ResMut<Server<(), ()>>, rt: Res<RuntimeResource>) {
	let addr = "127.0.0.1:8080";
	info!("Binding to {}", addr);
	in_order!(TEST: binding);
	server.bind(addr, rt.handle().clone());
	info!("Connecting to {}", addr);
	in_order!(TEST: connecting after binding);
	client.connect(addr, rt.handle().clone());
}

pub fn on_connect(mut connecteds: EventReader<FromClient<()>>, server: ResMut<Server<(), ()>>) {
	for event in connecteds.iter() {
		let Event::Connected(addr, id) = &**event else {
			continue
		};
		info!("Client {} connected: {:?}", id, &addr);
		in_order!(TEST: connected after connecting);
		server.connections.remove(&id);
	}
}

pub fn on_error(
	mut events: EventReader<FromServer<()>>,
	mut client: ResMut<Client<(), ()>>,
	mut exit: EventWriter<AppExit>,
) {
	for ev in events.iter() {
		let Event::Error(err, id) = &**ev else {
			continue
		};
		match err {
			ConnectionError::Disconnected => {
				info!("Client {} got disconnected.", id);
				in_order!(TEST: disconnected after connected);
				client.take().unwrap();
				info!("Shutting down app.");
				in_order!(TEST: shutdown after disconnected);
				exit.send(AppExit);
			}
			_ => {
				dbg!(err);
			}
		}
	}
}
