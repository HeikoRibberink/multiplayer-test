use std::marker::PhantomData;

use bevy::prelude::{Deref, DerefMut, EventWriter, Plugin, Res, Resource};
use serde::{Deserialize, Serialize};

use crate::{connection::ext::Event, NetStage};

use super::ServerHandle;

#[derive(Default, Debug)]
pub struct ServerPlugin<S, R>
where
	S: Serialize + Send + Sync + 'static,
	R: for<'de> Deserialize<'de> + Send + Sync + 'static,
{
	_s: PhantomData<S>,
	_r: PhantomData<R>,
}

impl<S, R> Plugin for ServerPlugin<S, R>
where
	S: Serialize + Send + Sync + 'static,
	R: for<'de> Deserialize<'de> + Send + Sync + 'static,
{
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_system_to_stage(NetStage::Receive, Server::<S, R>::event_system)
			.insert_resource(Server::<S, R>(ServerHandle::new()))
			.add_event::<FromClient<R>>();
	}
}

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct Server<S, R>(ServerHandle<S, R>)
where
	S: Serialize + Send + Sync + 'static,
	R: for<'de> Deserialize<'de> + Send + Sync + 'static;

#[derive(Deref, DerefMut)]
pub struct FromClient<R>(Event<R>);

impl<R> From<Event<R>> for FromClient<R> {
	fn from(value: Event<R>) -> Self {
		Self(value)
	}
}

impl<S, R> Server<S, R>
where
	S: Serialize + Send + Sync + 'static,
	R: for<'de> Deserialize<'de> + Send + Sync + 'static,
{
	pub fn event_system(server: Res<Server<S, R>>, mut eventwriter: EventWriter<FromClient<R>>) {
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
