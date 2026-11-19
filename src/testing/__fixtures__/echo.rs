use bevy::prelude::Message;
use derive_more::Deref;

use crate::testing;

#[derive(Debug, Clone, Deref, PartialEq, Eq, Message)]
pub struct Echo(pub u32);

impl From<&testing::CounterOutput> for Option<Echo> {
    fn from(output: &testing::CounterOutput) -> Self {
        Some(Echo(**output))
    }
}
