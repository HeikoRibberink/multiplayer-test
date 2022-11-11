pub mod bundle;
pub mod commands;
pub mod components;

use bevy::prelude::*;
use bevy::reflect as bevy_reflect;
use bevy::utils::Uuid;
use reflect_steroids::prelude::*;
use thiserror::Error;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

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

#[reflect_trait]
pub trait NetMsg: DowncastReflect {}

impl_dynamic_trait!(NetMsg, ReflectNetMsg);

/// Indicates that the server is allowed to send this message, and that the client should be ready to receive it.
// #[typetag::serde]
#[reflect_trait]
pub trait ToClientMsg: NetMsg {}

impl_dynamic_trait!(ToClientMsg, ReflectToClientMsg);

/// Indicates that the client is allowed to send this message, and that the server should be ready to receive it.
// #[typetag::serde]
#[reflect_trait]
pub trait ToServerMsg: NetMsg {}

impl_dynamic_trait!(ToServerMsg, ReflectToServerMsg);

/// Cast a type from and to a [NetMsg]
pub trait CastNetMsg {
	type Target: ToServerMsg + ToClientMsg + Clone;
	fn into_net_msg(&self) -> Self::Target;
	fn from_net_msg(from: Self::Target) -> Self;
}

#[derive(Error, Debug)]
pub enum SendError {
	#[error("IO Error")]
	IOError(#[from] tokio::io::Error),
	#[error("Unable to serialize message.")]
	SerializationError(#[from] postcard::Error),
}

#[derive(Error, Debug)]
pub enum RecvError {
	#[error("IO Error")]
	IOError(#[from] tokio::io::Error),
	#[error("Unable to deserialize payload.")]
	DeserializationError(#[from] postcard::Error),
}

pub(crate) async fn send_msg<W>(writer: &mut W, data: Vec<u8>) -> Result<(), SendError>
where
	W: AsyncWrite + Unpin,
{
	writer.write_u64_le(data.len() as u64).await?;
	writer.write_all(&*data).await?;
	Ok(())
}

pub(crate) async fn recv_msg<R>(reader: &mut R) -> Result<Vec<u8>, SendError>
where
	R: AsyncRead + Unpin,
{
	let num_bytes = reader.read_u64_le().await?;
	let mut buf = vec![0u8; num_bytes as usize];
	reader.read_exact(&mut *buf).await?;
	Ok(buf)
}
