use bevy::prelude::*;

use crate::{client::Client, NetEntityRegistry};

use super::{CastNetMsg, NetMsg, ToClientMsg, ToServerMsg, NetUuid};

#[derive(Reflect)]
pub struct SynchronizeEvent<T: Component + CastNetMsg> {
	msg: <T as CastNetMsg>::Target,
	uuid: NetUuid,
}

impl<T: Component + CastNetMsg> NetMsg for SynchronizeEvent<T> {}

impl<T: Component + CastNetMsg> ToServerMsg for SynchronizeEvent<T> {}

impl<T: Component + CastNetMsg> ToClientMsg for SynchronizeEvent<T> {}

#[derive(Component)]
pub struct Synchronize<T: Component + CastNetMsg> {
	inner: T,
	changed_only_by_recv: bool,
}

impl<T: Component + CastNetMsg> From<T> for Synchronize<T> {
    fn from(from: T) -> Self {
		Self {
			inner: from,
			changed_only_by_recv: false,
		}
    }
}

impl<T: Component + CastNetMsg> std::ops::Deref for Synchronize<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
		&self.inner
    }
}

impl<T: Component + CastNetMsg> std::ops::DerefMut for Synchronize<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
		self.changed_only_by_recv = false;
		&mut self.inner
    }
}

pub fn send_synchronize<T: Component + CastNetMsg>(query: Query<&Synchronize<T>, Changed<T>>, client: Res<Client>) {
	for i in query.iter() {
		//Prevents changes by recv_synchronize to be detected by this system, as this would be unnecessary work
		if i.changed_only_by_recv {continue;}
		client.send_msg(Box::new(i.clone().into_net_msg())).unwrap();
	}
}

pub fn recv_synchronize<T: Component + CastNetMsg>(
	mut query: Query<&mut Synchronize<T>>,
	local_entities: Res<NetEntityRegistry>,
	mut event_reader: EventReader<SynchronizeEvent<T>>,
) {
	for event in event_reader.iter() {
		let comp = <T as CastNetMsg>::from_net_msg(event.msg.clone());
		let entity = local_entities.get(event.uuid.into()).expect("Entity should be registered before synchronizing.");
		let mut sync = query.get_mut(entity).unwrap();
		sync.inner = comp;
		sync.changed_only_by_recv = true;
	}
}
