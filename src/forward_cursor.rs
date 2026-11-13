use std::marker::PhantomData;

use bevy::ecs::message::MessageCursor;
use bevy::prelude::{Message, Resource};

#[derive(Resource)]
pub struct ForwardCursor<M, E: Message> {
    pub(crate) cursor: MessageCursor<E>,
    marker: PhantomData<fn() -> M>,
}

impl<M, E: Message> Default for ForwardCursor<M, E> {
    fn default() -> Self {
        Self {
            cursor: MessageCursor::default(),
            marker: PhantomData,
        }
    }
}

// no test_usage necessary
