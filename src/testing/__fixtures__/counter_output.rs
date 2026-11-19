use bevy::prelude::Message;
use derive_more::Deref;

#[derive(Debug, Clone, Deref, PartialEq, Eq, Message)]
pub struct CounterOutput(pub u32);
