use bevy::prelude::{MessageWriter, Res, ResMut};

use crate::Now;
use crate::traits::PeriodicStateMachine;

pub fn tick_state_machine<M: PeriodicStateMachine>(
    mut machine: ResMut<M>,
    mut outputs: MessageWriter<M::Output>,
    now: Res<Now<M::Now>>,
) {
    let now = now.into_inner();
    outputs.write_batch(machine.tick(**now));
}
