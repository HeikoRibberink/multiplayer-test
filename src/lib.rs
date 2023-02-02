use bevy::{prelude::*, utils::Uuid};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use rand::Rng;
use tokio::runtime::Runtime;

pub mod client;
pub mod connection;
pub mod messaging;
pub mod server;

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system(register_type_data)
		.add_stage_before(
			CoreStage::Update,
			NetStage::Receive,
			SystemStage::parallel(),
		)
		.add_stage_after(CoreStage::Update, NetStage::Send, SystemStage::parallel())
		.insert_resource(NetEntityRegistry::default());
	}
}

pub static TYPE_REGISTRY: OnceCell<AppTypeRegistry> = OnceCell::new();

fn register_type_data(type_registry: Res<AppTypeRegistry>) {
	if let Err(_err) = TYPE_REGISTRY.set(type_registry.clone()) {
		panic!("This system should be the only system to set the type registry.");
	}
}

#[derive(StageLabel)]
pub enum NetStage {
	Receive,
	Send,
}

pub type EntityUuid = Uuid;

#[derive(Resource, Default)]
pub struct NetEntityRegistry {
	map: DashMap<EntityUuid, Entity>,
}

impl NetEntityRegistry {
	pub fn register(&self, entity: Entity) -> EntityUuid {
		let rand_uuid = Uuid::from_bytes(rand::thread_rng().gen::<_>());
		self.map.insert(rand_uuid, entity).expect("uuid clash");
		rand_uuid
	}

	pub fn deregister(&self, uuid: EntityUuid) -> Option<(EntityUuid, Entity)> {
		self.map.remove(&uuid)
	}

	pub fn get(&self, uuid: EntityUuid) -> Option<Entity> {
		if let Some(val) = self.map.get(&uuid) {
			Some(*val.value())
		} else {
			None
		}
	}
}

#[derive(Deref, Debug, Resource)]
pub struct RuntimeResource(pub Runtime);