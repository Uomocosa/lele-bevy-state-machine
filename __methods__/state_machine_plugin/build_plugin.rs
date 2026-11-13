use bevy::ecs::schedule::ScheduleLabel;
#[cfg(feature = "testing")]
use bevy::prelude::not;
use bevy::prelude::{App, First, IntoScheduleConfigs, Messages, Update, resource_exists};

use crate::Now;
use crate::bevy_systems;
use crate::state_machine_plugin::StateMachinePlugin;
#[cfg(feature = "testing")]
use crate::testing;
use crate::traits::{Clock, StateMachine};

pub fn build_plugin<M: StateMachine>(plugin: &StateMachinePlugin<M>, app: &mut App) {
    app.init_resource::<Messages<M::Input>>();
    app.add_message::<M::Output>();
    if !app.world().contains_resource::<Now<M::Now>>() {
        app.insert_resource(Now(M::Now::now()));
        #[cfg(feature = "testing")]
        app.add_systems(
            First,
            bevy_systems::refresh_now::<M::Now>
                .run_if(not(resource_exists::<testing::FrozenClock>)),
        );
        #[cfg(not(feature = "testing"))]
        app.add_systems(First, bevy_systems::refresh_now::<M::Now>);
    }
    StateMachinePlugin::<M>::configure_update_schedule::<M>(app, Update.intern());
    for add_extra_schedule in &plugin.extra_update_schedules {
        add_extra_schedule(app);
    }
    if let Some(add_periodic_systems) = plugin.add_periodic_systems {
        add_periodic_systems(app);
    }
    for adapter in &plugin.adapters {
        adapter(app);
    }
}

// no test_usage necessary
