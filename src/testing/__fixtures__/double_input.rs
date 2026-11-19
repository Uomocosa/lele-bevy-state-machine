use bevy::prelude::Message;
use derive_more::Deref;

#[derive(Message, Deref)]
pub struct DoubleInput(pub u32);
