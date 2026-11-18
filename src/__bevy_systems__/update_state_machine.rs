use bevy::prelude::{MessageWriter, Messages, Res, ResMut};

use crate::Now;
use crate::traits::StateMachine;

pub fn update_state_machine<M: StateMachine>(
    mut machine: ResMut<M>,
    mut inputs: ResMut<Messages<M::Input>>,
    mut outputs: MessageWriter<M::Output>,
    now: Res<Now<M::Now>>,
) {
    let now = now.into_inner();
    for input in inputs.drain() {
        outputs.write_batch(machine.update(input, **now));
    }
}
