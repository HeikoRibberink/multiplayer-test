#![feature(auto_traits)]

use std::marker::PhantomData;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod networking;
pub mod client;

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
		
	}
}
