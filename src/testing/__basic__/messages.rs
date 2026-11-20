use bevy::prelude::Message;
use derive_more::Deref;

#[derive(Message)]
pub struct EmptyInput;

#[derive(Message)]
pub struct EmptyOutput;

#[derive(Message, Deref)]
pub struct DoubleInput(pub u32);

#[derive(Message, Debug, Deref, PartialEq, Eq)]
pub struct DoubleOutput(pub u32);

#[derive(Debug, Clone, Deref, PartialEq, Eq, Message)]
pub struct CounterOutput(pub u32);
