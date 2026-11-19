use bevy::prelude::Message;
use derive_more::Deref;

use crate::testing;

#[derive(Debug, Clone, Deref, PartialEq, Eq, Message)]
pub struct Mirror(pub u32);

impl From<&testing::CounterOutput> for Option<Mirror> {
    fn from(output: &testing::CounterOutput) -> Self {
        (**output > 3).then_some(Mirror(**output))
    }
}
