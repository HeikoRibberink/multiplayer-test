use std::net::SocketAddr;

use bevy::prelude::EventWriter;
use serde::{Deserialize, Serialize};

use super::{ConnectionError, ConnectionHandle, ConnectionId};

#[derive(Debug)]
pub enum Event<R> {
	Message(R, ConnectionId),
	Connected(SocketAddr, ConnectionId),
	Error(ConnectionError, ConnectionId),
}

impl<R> From<(SocketAddr, ConnectionId)> for Event<R> {
	fn from(value: (SocketAddr, ConnectionId)) -> Self {
		Self::Connected(value.0, value.1)
	}
}

pub fn emit_messages_as_events<S, R>(
	handle: &ConnectionHandle<S, R>,
	eventwriter: &mut EventWriter<Event<R>>,
) where
	S: Serialize + Send + 'static,
	for<'de> R: Deserialize<'de> + Send + Sync + 'static,
{
	loop {
		let recv = handle.try_recv();
		if let Ok(opt) = recv {
			let Some(val) = opt else {
				break;
			};
			eventwriter.send(Event::Message(val, handle.uuid));
		} else if let Err(err) = recv {
			eventwriter.send(Event::Error(err, handle.uuid));
			break;
		}
	}
}
