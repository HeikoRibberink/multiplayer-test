use bevy::prelude::*;
use bevy::utils::Uuid;

pub mod bundles; // A Bundle of Components, which is basically an entity to be spawned.
pub mod netfields;
pub mod procedures; // Custom procedures // All possible networked Components

//Every operation is defined as a message, like in Smalltalk
//When an operation is synced, it should happen through the network implementation
//and should be registered by the network implementation
//e.g.: we spawn a synced entity: the network implementation needs to keep track of the networked entities

///Every generic parameter should be an enum containing all the possibles types for that kind.
pub enum Msg<Bundles, Procedures, Settables> {
	Spawn(Bundles), // On spawn, the server should register the spawned entity in a Map<Uuid, Entity>.
	Despawn(Uuid),  // On despawn, the server should deregister the specifed entity from the registry.
	Command(Procedures),
	Set(Uuid, Settables), //This should allow specific fields (or whole components) on a component of an entity to be set.
}

#[derive(Clone, Copy, Debug)]
pub struct NetworkedEntity {
	pub(crate) local: Entity,
	pub(crate) network: Uuid,
}

impl PartialEq<NetworkedEntity> for NetworkedEntity {
	fn eq(&self, other: &NetworkedEntity) -> bool {
		self.network == other.network
	}
}

impl From<NetworkedEntity> for Uuid {
	fn from(n: NetworkedEntity) -> Self {
		n.network
	}
}

mod tests {
	use bevy::prelude::{Bundle, Transform};
	use serde::{Deserialize, Serialize};
	use super::{Msg};
use super::bundles::NetBundle;
	use super::bundles::macros::NetBundle;

	#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
	enum SizeTest {
		Foo(i32),
		Bar(u128, u128),
	}

	#[test]
	fn size_test() {
		use postcard::*;
		let foo = SizeTest::Foo(300000);
		let bar = SizeTest::Bar(1000000000000000000000, 1000000000000000000);
		let f = to_stdvec(&foo).unwrap();
		let b = to_stdvec(&bar).unwrap();
		println!("{f:?}, {b:?}");
		assert_ne!(f.len(), b.len());
		assert_eq!(from_bytes::<SizeTest>(&f).unwrap(), foo);
		assert_eq!(from_bytes::<SizeTest>(&b).unwrap(), bar);
	}

	#[derive(Bundle, NetBundle, Default)]
	struct TestBundle {
		#[networked]
		transform: Transform,
	}

	enum Bundles {
		TestBundle(TestBundle),
	}


	fn test_netbundlemsg_trait() {
	}
}
