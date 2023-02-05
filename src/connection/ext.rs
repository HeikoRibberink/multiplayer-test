use std::net::SocketAddr;

use super::{ConnectionError, ConnectionId};

#[derive(Debug)]
pub enum Event<R> {
	Message(R, ConnectionId),
	Connected(SocketAddr, ConnectionId),
	Error(ConnectionError, ConnectionId),
}

impl<R> Event<R> {
	pub fn unwrap_message(self) -> (R, ConnectionId) {
		match self {
			Event::Message(recv, id) => (recv, id),
			Event::Connected(_, _) => panic!("Expected `Event::Message`, got `Event::Connected`."),
			Event::Error(_, _) => panic!("Expected `Event::Message`, got `Event::Error`."),
		}
	}
}

impl<R> From<(SocketAddr, ConnectionId)> for Event<R> {
	fn from(value: (SocketAddr, ConnectionId)) -> Self {
		Self::Connected(value.0, value.1)
	}
}
