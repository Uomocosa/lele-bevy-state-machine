use bevy::prelude::{Message, MessageWriter, Messages, Res, ResMut};

use crate::forward_cursor::ForwardCursor;
use crate::traits::StateMachine;

pub fn forward_input<M: StateMachine, E: Message>(
    mut forwarded: ResMut<ForwardCursor<M, E>>,
    events: Res<Messages<E>>,
    mut inputs: MessageWriter<M::Input>,
) where
    for<'e> M::Input: From<&'e E>,
{
    let events = events.into_inner();
    inputs.write_batch(forwarded.cursor.read(events).map(M::Input::from));
}
