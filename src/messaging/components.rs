use std::marker::PhantomData;

use bevy::prelude::*;

use super::{CastNetMsg, NetUuid, NetMsg};

#[derive(Reflect)]
pub struct SynchronizeEvent<T: Component + CastNetMsg> {
	msg: T::Target,
	uuid: NetUuid,
}

// impl<T: Component + CastNetMsg> NetMsg for SynchronizeEvent<T> {}

// impl<T: Component + CastNetMsg> ToServerMsg for SynchronizeEvent<T> {}

// impl<T: Component + CastNetMsg> ToClientMsg for SynchronizeEvent<T> {}

#[derive(Component)]
pub struct Synchronize<T: Component + CastNetMsg> {
	_m: PhantomData<T>,
	// changed_only_by_recv: bool,
}

impl<T: Component + CastNetMsg> Synchronize<T> {
	pub fn new() -> Self {
		Self {
			_m: PhantomData,
		// changed_only_by_recv: false,
		}
	}
}

// pub fn send_synchronize<T: Component + CastNetMsg>(query: Query<&T, (With<Synchronize<T>>, Changed<T>)>, client: Res<Client>) {
// 	for t in query.iter() {
// 		// //Prevents changes by recv_synchronize to be detected by this system, as this would be unnecessary work
// 		// if sync.changed_only_by_recv {continue;}
// 		client.send_msg(Box::new(t.into_net_msg())).unwrap();
// 	}
// }

// pub fn recv_synchronize<T: Component + CastNetMsg>(
// 	mut query: Query<&mut T, With<Synchronize<T>>>,
// 	local_entities: Res<NetEntityRegistry>,
// 	mut event_reader: EventReader<SynchronizeEvent<T>>,
// ) {
// 	for event in event_reader.iter() {
// 		let entity = local_entities.get(event.uuid.into()).expect("Entity should be registered before synchronizing.");
// 		let mut t = query.get_mut(entity).unwrap();
// 		t.set_with_net_msg(event.msg.clone());
// 		// sync.changed_only_by_recv = true;
// 	}
// }
