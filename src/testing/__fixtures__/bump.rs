use bevy::prelude::Message;
use derive_more::Deref;

#[derive(Debug, Clone, Deref, PartialEq, Eq, Message)]
pub struct Bump(pub u32);
