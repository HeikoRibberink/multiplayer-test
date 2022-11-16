use bevy::{prelude::EventWriter, utils::Uuid};
use serde::{Deserialize, Serialize};

use super::{ConnectionError, ConnectionHandle};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConnectionEvent<R>(pub R, pub Uuid);

pub type ConnErrorEvent = ConnectionEvent<ConnectionError>;

pub fn emit_messages_as_events<S, R>(
	handle: &ConnectionHandle<S, R>,
	data_events: &mut EventWriter<ConnectionEvent<R>>,
	error_events: &mut EventWriter<ConnectionEvent<ConnectionError>>,
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
			data_events.send(ConnectionEvent(val, handle.uuid));
		} else if let Err(err) = recv {
			error_events.send(ConnectionEvent(err, handle.uuid));
			break;
		}
	}
}
