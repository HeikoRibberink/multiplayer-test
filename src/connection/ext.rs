use std::net::SocketAddr;

use bevy::prelude::EventWriter;
use serde::{Deserialize, Serialize};

use super::{ConnectionError, ConnectionHandle, ConnectionId};

#[derive(Debug)]
pub struct ErrorEvent(pub ConnectionError, pub ConnectionId);
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConnectedEvent(pub SocketAddr, pub ConnectionId);
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MessageEvent<R>(pub R, pub ConnectionId);

pub fn emit_messages_as_events<S, R>(
	handle: &ConnectionHandle<S, R>,
	connection_messages: &mut EventWriter<MessageEvent<R>>,
	connection_errors: &mut EventWriter<ErrorEvent>,
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
			connection_messages.send(MessageEvent(val, handle.uuid));
		} else if let Err(err) = recv {
			connection_errors.send(ErrorEvent(err, handle.uuid));
			break;
		}
	}
}
