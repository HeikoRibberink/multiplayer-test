// use bevy::{ecs::system::Command, prelude::*};

// use super::bundle::NetBundle;

// pub trait SpawnableBundle: Bundle + Struct + Reflect {}

// pub struct SpawnNetBundle {
// 	bundle: Box<dyn SpawnableBundle>,
// }

// impl SpawnNetBundle {
// 	pub fn new(bundle: impl NetBundle) -> Self {
// 		Self {
// 			bundle: Box::new(bundle),
// 		}
// 	}
// }

// impl Command for SpawnNetBundle {
// 	fn write(self, world: &mut World) {
// 		self.bundle as &dyn Bundle
// 	}
// }

use bevy::{prelude::*, ecs::system::EntityCommands};

pub trait NetCommands<'w, 's> {
	fn spawn_net<'a>(&'a mut self) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> NetCommands<'w, 's> for Commands<'w, 's> {
	fn spawn_net<'a>(&'a mut self) -> EntityCommands<'w, 's, 'a> {
		
		self.spawn_empty()
	}
}