use bevy::prelude::{Message, MessageWriter, Messages, Res, ResMut};

use crate::forward_cursor::ForwardCursor;
use crate::traits::StateMachine;

pub fn forward_output<M: StateMachine, T: Message>(
    mut cursor: ResMut<ForwardCursor<(M, T), M::Output>>,
    outputs: Res<Messages<M::Output>>,
    mut forwarded: MessageWriter<T>,
) where
    for<'o> Option<T>: From<&'o M::Output>,
{
    let outputs = outputs.into_inner();
    forwarded.write_batch(cursor.cursor.read(outputs).filter_map(Option::<T>::from));
}
