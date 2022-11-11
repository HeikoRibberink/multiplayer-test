use std::marker::PhantomData;

use bevy::{prelude::*, utils::Uuid};
use client::Client;
use rand::Rng;
use serde::{Deserialize, Serialize};
use dashmap::DashMap;

pub mod client;
pub mod messaging;
pub mod server;

#[derive(Default)]
struct MultiplayerPlugin<T>
where
	for<'a> T: Send + Sync + Serialize + Deserialize<'a> + 'static,
{
	_m: PhantomData<T>,
}

impl<T> Plugin for MultiplayerPlugin<T>
where
	for<'a> T: Send + Sync + Serialize + Deserialize<'a> + 'static,
{
	fn build(&self, app: &mut App) {
		app
		.add_stage_before(CoreStage::Update, NetStage::Receive, SystemStage::parallel())
		.add_stage_after(CoreStage::Update, NetStage::Send, SystemStage::parallel())
		.insert_resource(NetEntityRegistry::default())
		;
	}
}


#[derive(StageLabel)]
pub enum NetStage {
	Receive,
	Send,
}

#[derive(Default)]
pub struct NetEntityRegistry {
	map: DashMap<Uuid, Entity>,
}

impl NetEntityRegistry {
	pub fn register(&self, entity: Entity) -> Uuid {
		let rand_uuid = Uuid::from_bytes(rand::thread_rng().gen::<_>());
		self.map.insert(rand_uuid, entity).expect("uuid clash");
		rand_uuid
	}

	pub fn deregister(&self, uuid: Uuid) -> Option<(Uuid, Entity)> {
		self.map.remove(&uuid)
	}

	pub fn get(&self, uuid: Uuid) -> Option<Entity> {
		if let Some(val) = self.map.get(&uuid) {
			Some(*val.value())
		} else {
			None
		}
	}
}
