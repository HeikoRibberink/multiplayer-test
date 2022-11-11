use bevy::prelude::*;
use bevy::reflect as bevy_reflect;
use reflect_steroids::{impl_dynamic_trait, DowncastReflect};

#[reflect_trait]
pub unsafe trait NetBundle: Struct + DowncastReflect {}

unsafe impl<T> NetBundle for T where T: Bundle + Struct + DowncastReflect {}

impl_dynamic_trait!(NetBundle, ReflectNetBundle);

//TODO: make a derive macro for automatically converting e.g. a Spawn(Bundle) into an action on the client.
//DON'T TRY TO AUTOMATICALLY SPAWN ANY `&dyn NetBundle`, IT IS WAY TO CONVOLUTED AND DIFFICULT AND YOU SHOULD PREFER HAVING A SYSTEM FOR EVERY BUNDLE WHICH CAN AUTOMATICALLY DOWNCAST IT
