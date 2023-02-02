pub mod bundle;
pub mod commands;
pub mod components;

use std::ops::Deref;

use bevy::{
	prelude::*,
	reflect::serde::{ReflectSerializer, UntypedReflectDeserializer},
	utils::Uuid,
};
use serde::{de::DeserializeSeed, Deserialize, Serialize, __private::de};
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::TYPE_REGISTRY;

/// Only use for sending over the network
#[derive(Reflect, Clone, Copy, Component, PartialEq, Eq, Hash)]
pub struct NetUuid([u8; 16]);

impl From<Uuid> for NetUuid {
	fn from(uuid: Uuid) -> Self {
		Self(uuid.into_bytes())
	}
}

impl From<NetUuid> for Uuid {
	fn from(net: NetUuid) -> Self {
		Self::from_bytes(net.0)
	}
}

#[derive(Debug, Deref, DerefMut)]
pub struct NetMsg {
	pub inner: Box<dyn Reflect>,
}

impl From<Box<dyn Reflect>> for NetMsg {
	fn from(v: Box<dyn Reflect>) -> Self {
		Self { inner: v }
	}
}

impl Serialize for NetMsg {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let binding = TYPE_REGISTRY
			.get()
			.expect("The global type registry should be available before this can run.")
			.0
			.read();
		let type_registry = binding.deref();
		let ser = ReflectSerializer::new(self.inner.as_reflect(), type_registry);
		Ok(ser.serialize(serializer)?)
	}
}

impl<'de> Deserialize<'de> for NetMsg {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let binding = TYPE_REGISTRY
			.get()
			.expect("The global type registry should be available before this can run.")
			.0
			.read();
		let type_registry = binding.deref();
		let de = UntypedReflectDeserializer::new(type_registry);
		Ok(de.deserialize(deserializer)?.into())
	}
}

/// Cast a type from and to a [NetMsg]
pub trait CastNetMsg: Sized {
	type Target: Reflect;
	fn extract_to_target(&self) -> Self::Target;
	fn create_from_target(from: Self::Target) -> Self;
	fn set_with_target(&mut self, from: Self::Target);
	fn into_netmsg(&self) -> NetMsg {
		let boxed = Box::new(self.extract_to_target());
		let reflect = boxed.into_reflect();
		reflect.into()
	}
	fn from_netmsg(from: NetMsg) -> Result<Self, NetMsg> {
		if !from.is::<Self::Target>() {
			return Err(from);
		}
		// let target: Box<Self::Target> = from.inner.downcast().ok().;
		// Self::create_from_target(target.)
		todo!()
	}
}

pub(crate) async fn send_msg<W>(writer: &mut W, data: Vec<u8>) -> Result<(), io::Error>
where
	W: AsyncWrite + Unpin,
{
	writer.write_u64_le(data.len() as u64).await?;
	writer.write_all(&*data).await?;
	Ok(())
}

pub(crate) async fn recv_msg<R>(reader: &mut R) -> Result<Vec<u8>, io::Error>
where
	R: AsyncRead + Unpin,
{
	let num_bytes = reader.read_u64_le().await?;
	let mut buf = vec![0u8; num_bytes as usize];
	reader.read_exact(&mut *buf).await?;
	Ok(buf)
}
