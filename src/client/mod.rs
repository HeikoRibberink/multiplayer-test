use std::{marker::{PhantomData, Send, Sync}, net::{SocketAddrV4, Ipv4Addr}};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::net::ToSocketAddrs;

use tokio::runtime::Handle;

use crate::{
	connection::{ext::Event, ConnectionHandle},
	NetStage,
};

#[derive(Default, Debug)]
pub struct ClientPlugin<S, R>
where
	S: Serialize + Send + 'static,
	R: for<'de> Deserialize<'de> + Send + 'static,
{
	_s: PhantomData<S>,
	_r: PhantomData<R>,
}

impl<S, R> Plugin for ClientPlugin<S, R>
where
	S: Serialize + Send + Sync + 'static,
	R: for<'de> Deserialize<'de> + Send + Sync + 'static,
{
	fn build(&self, app: &mut App) {
		app.add_system_to_stage(NetStage::Receive, Client::<S, R>::event_system)
			.insert_resource(Client::<S, R>(None))
			.add_event::<FromServer<R>>();
	}
}

#[derive(Debug, Deref, DerefMut)]
pub struct FromServer<R>(pub Event<R>);

impl<R> From<Event<R>> for FromServer<R> {
	fn from(value: Event<R>) -> Self {
		Self(value)
	}
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct Client<S, R>(Option<ConnectionHandle<S, R>>)
where
	S: Serialize + Send + Sync + 'static,
	R: for<'de> Deserialize<'de> + Send + Sync + 'static;

impl<S, R> Client<S, R>
where
	S: Serialize + Send + Sync + 'static,
	R: for<'de> Deserialize<'de> + Send + Sync + 'static,
{
	pub fn connect<A>(&mut self, addr: A, rt: Handle)
	where
		A: ToSocketAddrs + Send + 'static,
	{
		self.0 = Some(ConnectionHandle::connect(addr, rt));
	}
	pub fn event_system(client: Res<Client<S, R>>, mut eventwriter: EventWriter<FromServer<R>>, mut first_msg: Local<bool>) {
		let Some(client) = &**client else {
			return
		};
		if !*first_msg {
			*first_msg = true;
			eventwriter.send(Event::Connected(std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)), client.uuid).into())
		}
		loop {
			let recv = client.try_recv();
			if let Ok(opt) = recv {
				let Some(val) = opt else {
					break;
				};
				eventwriter.send(Event::Message(val, client.uuid).into());
			} else if let Err(err) = recv {
				eventwriter.send(Event::Error(err, client.uuid).into());
				break;
			}
		}
	}
}
