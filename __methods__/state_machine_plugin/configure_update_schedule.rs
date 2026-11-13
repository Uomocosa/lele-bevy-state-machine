use bevy::ecs::schedule::InternedScheduleLabel;
use bevy::prelude::{App, IntoScheduleConfigs, resource_exists};

use crate::bevy_systems;
use crate::state_machine_systems::StateMachineSystems;
use crate::traits::StateMachine;

pub fn configure_update_schedule<Machine: StateMachine>(
    app: &mut App,
    schedule: InternedScheduleLabel,
) {
    app.configure_sets(
        schedule,
        (
            StateMachineSystems::collect_inputs::<Machine>(),
            StateMachineSystems::apply_inputs::<Machine>(),
            StateMachineSystems::perform_outputs::<Machine>(),
        )
            .chain(),
    );
    app.add_systems(
        schedule,
        bevy_systems::update_state_machine::<Machine>
            .in_set(StateMachineSystems::apply_inputs::<Machine>())
            .run_if(resource_exists::<Machine>),
    );
}

// no test_usage necessary
