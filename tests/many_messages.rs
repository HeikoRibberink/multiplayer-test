#![cfg(test)]
use bevy::log::{Level, LogPlugin};
use bevy::{app::AppExit, prelude::*};
use multiplayer_test::connection::ConnectionHandle;
use multiplayer_test::connection::ext::{ConnectedEvent, MessageEvent};
use multiplayer_test::messaging::NetMsg;
use multiplayer_test::server::{Server, ServerHandle, ServerPlugin};
use multiplayer_test::{
	self,
	client::{Client, ClientPlugin},
	connection::{ext::ErrorEvent, ConnectionError},
	MultiplayerPlugin, RuntimeResource,
};
use assert_in_order::*;

in_order_init!(TEST);

#[test]
fn many_messages() -> Result<(), Box<dyn std::error::Error>> {
	let rt = tokio::runtime::Builder::new_multi_thread()
		.worker_threads(1)
		.enable_io()
		.build()?;

	App::new()
		.add_plugins(MinimalPlugins)
		.add_plugin(LogPlugin {
			level: Level::INFO,
			..default()
		})
		.add_plugin(MultiplayerPlugin)
		.add_plugin(ClientPlugin)
		.add_plugin(ServerPlugin)
		.insert_resource(RuntimeResource(rt))
		.add_startup_system(setup)
		.add_system(on_error)
		.add_system(on_connect)
		.run();
	Ok(())
}

pub fn setup(mut client: ResMut<Client>, mut server: ResMut<Server>, rt: Res<RuntimeResource>) {
	let addr = "127.0.0.1:8080";
	info!("Binding to {}", addr);
	in_order!(TEST: binding);
	server.server = Some(ServerHandle::bind(rt.handle().clone(), addr));
	info!("Connecting to {}", addr);
	in_order!(TEST: connecting after binding);
	client.connection = Some(ConnectionHandle::connect(rt.handle().clone(), addr));
	// client.connection.as_mut().unwrap().send_blocking()
}

pub fn on_connect(mut events: EventReader<ConnectedEvent>, mut server: ResMut<Server>) {
	for event in events.iter() {
		println!("Client connected: {:?}", event.0);
		in_order!(TEST: connected after connecting);
	}
}

pub fn on_msg(
	mut events: EventReader<MessageEvent<NetMsg>>,
	mut client: ResMut<Client>,
) {
	for event in events.iter() {
		info!("Msg received: {:?}", event);
	}
}

pub fn on_error(
	mut events: EventReader<ErrorEvent>,
	mut client: ResMut<Client>,
	mut exit: EventWriter<AppExit>,
) {
	for ev in events.iter() {
		match ev.0 {
			ConnectionError::Disconnected => {
				// info!("Client got disconnected.");
				in_order!(TEST: disconnected after connected);
				client.connection.take().unwrap();
				// info!("Shutting down app.");
				in_order!(TEST: shutdown after disconnected);
				exit.send(AppExit);
			}
			_ => {
				dbg!(ev);
			}
		}
	}
}
