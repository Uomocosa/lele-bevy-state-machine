use bevy::prelude::Message;

use crate::testing;

#[derive(Debug, Clone, PartialEq, Eq, Message)]
pub enum CounterInput {
    Add(u32),
}

#[rustfmt::skip]
impl From<&testing::Bump> for CounterInput {
    fn from(bump: &testing::Bump) -> Self { Self::Add(**bump) }
}
