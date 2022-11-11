#![cfg(test)]
use bevy::prelude::*;
use multiplayer_test::{
	self,
	client::{Client, ClientPlugin, FromClient},
};
use tokio::net::TcpListener;
#[test]
fn basic_connection() -> Result<(), Box<dyn std::error::Error>> {
	let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_io().build()?;
	let _handle = rt.spawn(async {
		let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
		while let Ok((_stream, addr)) = listener.accept().await {
			println!("Connected: {}", addr);
		}
	});

	App::new()
		.add_plugins(DefaultPlugins)
		.insert_resource(rt)
		.add_plugin(ClientPlugin)
		.add_startup_system(setup)
		.add_system(on_connect)
		.run();

	Ok(())
}

pub fn setup(mut client: ResMut<Client>) {
	client.connect("127.0.0.1:8080".parse().unwrap()).unwrap();
}

pub fn on_connect(mut events: EventReader<FromClient>) {
	for ev in events.iter() {
		dbg!(ev);
		if let FromClient::Connected = ev {
			println!("connected");
		}
	}
}
