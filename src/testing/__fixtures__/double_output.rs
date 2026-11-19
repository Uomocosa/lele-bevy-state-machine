use bevy::prelude::Message;
use derive_more::Deref;

#[derive(Message, Debug, Deref, PartialEq, Eq)]
pub struct DoubleOutput(pub u32);
