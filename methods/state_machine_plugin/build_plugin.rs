#[cfg(feature = "testing")]
use bevy::prelude::not;
use bevy::prelude::{App, First, IntoScheduleConfigs, Last, Messages, Update, resource_exists};

use crate::basic;
use crate::bevy_systems;
use crate::clock::Clock;
use crate::state_machine::StateMachine;
use crate::state_machine_plugin::StateMachinePlugin;
use crate::state_machine_systems::StateMachineSystems;
#[cfg(feature = "testing")]
use crate::testing;

pub fn build_plugin<M: StateMachine>(plugin: &StateMachinePlugin<M>, app: &mut App) {
    app.init_resource::<Messages<M::Input>>();
    app.add_message::<M::Output>();
    if !app
        .world()
        .contains_resource::<basic::resources::Now<M::Now>>()
    {
        app.insert_resource(basic::resources::Now(M::Now::now()));
        #[cfg(feature = "testing")]
        app.add_systems(
            First,
            bevy_systems::refresh_now::<M::Now>
                .run_if(not(resource_exists::<testing::FrozenClock>)),
        );
        #[cfg(not(feature = "testing"))]
        app.add_systems(First, bevy_systems::refresh_now::<M::Now>);
    }
    app.configure_sets(
        Update,
        (
            StateMachineSystems::collect_inputs::<M>(),
            StateMachineSystems::step::<M>(),
            StateMachineSystems::perform_outputs::<M>(),
        )
            .chain(),
    );
    app.add_systems(
        Update,
        bevy_systems::step_state_machine::<M>
            .in_set(StateMachineSystems::step::<M>())
            .run_if(resource_exists::<M>),
    );
    if plugin.is_stepped_on_exit {
        app.configure_sets(
            Last,
            (
                StateMachineSystems::collect_inputs::<M>(),
                StateMachineSystems::step::<M>(),
                StateMachineSystems::perform_outputs::<M>(),
            )
                .chain(),
        );
        app.add_systems(
            Last,
            bevy_systems::step_state_machine::<M>
                .in_set(StateMachineSystems::step::<M>())
                .run_if(resource_exists::<M>),
        );
    }
    if let Some(add_tick_system) = plugin.add_tick_system {
        add_tick_system(app);
    }
}

// no test_usage necessary
