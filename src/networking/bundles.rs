//Option 1: define a `NetBundle` trait that has default fields (like texture etc), which are not send, and parameters which do get send.
use bevy::prelude::*;

pub mod macros {
	pub use multiplayer_test_macros::NetBundle;
}

pub trait NetBundle: Bundle + Default
{
	type NetComps: Clone + Send;
	fn get_networked(&self) -> Self::NetComps;
	fn from_networked(components: Self::NetComps) -> Self;
}

mod tests {
	use bevy::prelude::*;
	use multiplayer_test_macros::NetBundle;

	use super::NetBundle;

	#[test]
	fn test_nentity() {
		let a = NetworkedEntity {
			transform: Default::default(),
			global_transform: Default::default(),
		};
		let b = a.get_networked();
		let c = NetworkedEntity::from_networked(b);
		assert_eq!(a, c);
	}

	#[derive(NetBundle, Bundle, Default, PartialEq, Debug)]
	struct NetworkedEntity {
		#[networked]
		pub transform: Transform,
		// #[networked]
		pub global_transform: GlobalTransform,
	}
}
