use std::marker::PhantomData;

use atomic_delegate_macros::atomic_delegates;
use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use bevy::prelude::{App, First, IntoScheduleConfigs, Message, Messages, Plugin, Update};

#[cfg(feature = "testing")]
use bevy::prelude::{not, resource_exists};
use bevy::time::common_conditions::on_timer;

use crate::Now;
use crate::bevy_systems;
use crate::forward_cursor::ForwardCursor;
use crate::state_machine_systems::StateMachineSystems;
#[cfg(feature = "testing")]
use crate::testing;
use crate::traits::{Clock, PeriodicStateMachine, StateMachine};

pub struct StateMachinePlugin<M> {
    pub(crate) add_periodic_systems: Option<fn(&mut App)>,
    pub(crate) extra_update_schedules: Vec<Box<dyn Fn() -> InternedScheduleLabel + Send + Sync>>,
    pub(crate) adapters: Vec<fn(&mut App, InternedScheduleLabel)>,
    marker: PhantomData<fn() -> M>,
}

impl<M: StateMachine> Default for StateMachinePlugin<M> {
    fn default() -> Self {
        Self {
            add_periodic_systems: None,
            extra_update_schedules: Vec::new(),
            adapters: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<M: StateMachine> Plugin for StateMachinePlugin<M> {
    fn build(&self, app: &mut App) {
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
        let extra_schedules = self.extra_update_schedules.iter().map(|label| label());
        for schedule in std::iter::once(Update.intern()).chain(extra_schedules) {
            Self::configure_update_schedule::<M>(app, schedule);
            for adapter in &self.adapters {
                adapter(app, schedule);
            }
        }
        if let Some(add_periodic_systems) = self.add_periodic_systems {
            add_periodic_systems(app);
        }
    }
}

impl<M: StateMachine> StateMachinePlugin<M> {
    #[must_use]
    pub fn also_update_in(mut self, schedule: impl ScheduleLabel) -> Self {
        self.extra_update_schedules
            .push(Box::new(move || schedule.intern()));
        self
    }

    #[must_use]
    pub fn input_from<E: Message>(mut self) -> Self
    where
        for<'e> M::Input: From<&'e E>,
    {
        self.adapters.push(|app: &mut App, schedule| {
            app.init_resource::<ForwardCursor<M, E>>();
            app.add_systems(
                schedule,
                bevy_systems::forward_input::<M, E>
                    .in_set(StateMachineSystems::collect_inputs::<M>()),
            );
        });
        self
    }

    #[must_use]
    pub fn output_into<T: Message>(mut self) -> Self
    where
        for<'o> Option<T>: From<&'o M::Output>,
    {
        self.adapters.push(|app: &mut App, schedule| {
            app.init_resource::<ForwardCursor<(M, T), M::Output>>();
            app.add_systems(
                schedule,
                bevy_systems::forward_output::<M, T>
                    .in_set(StateMachineSystems::perform_outputs::<M>()),
            );
        });
        self
    }
}

impl<M: PeriodicStateMachine> StateMachinePlugin<M> {
    #[must_use]
    pub fn periodic(mut self) -> Self {
        self.add_periodic_systems = Some(|app: &mut App| {
            app.add_systems(
                Update,
                bevy_systems::tick_state_machine::<M>
                    .in_set(StateMachineSystems::apply_inputs::<M>())
                    .after(bevy_systems::update_state_machine::<M>)
                    .run_if(on_timer(M::TICK_INTERVAL)),
            );
        });
        self
    }
}

#[atomic_delegates]
impl<M: StateMachine> StateMachinePlugin<M> {
    pub(crate) fn configure_update_schedule<Machine: StateMachine>(
        app: &mut App,
        schedule: InternedScheduleLabel,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::AtomicU32;
    use std::time::Duration;

    use super::*;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use derive_more::Deref;

    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = testing::counter_app(StateMachinePlugin::default());
        app.world_mut()
            .resource_mut::<Messages<testing::CounterInput>>()
            .write(testing::CounterInput::Add(2));
        app.update();
        assert_eq!(
            testing::drain_messages::<testing::CounterOutput>(&mut app),
            vec![testing::CounterOutput(2)]
        );
    }

    #[test]
    fn inputs_written_before_machine_insert_are_not_lost() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StateMachinePlugin::<testing::CounterMachine>::default());
        app.world_mut()
            .resource_mut::<Messages<testing::CounterInput>>()
            .write(testing::CounterInput::Add(3));
        app.update();
        app.world_mut()
            .insert_resource(testing::CounterMachine(AtomicU32::new(0)));
        app.update();
        assert_eq!(
            testing::drain_messages::<testing::CounterOutput>(&mut app),
            vec![testing::CounterOutput(3)]
        );
    }

    #[test]
    fn outputs_readable_in_perform_outputs_same_frame() {
        #[derive(Resource, Default, Deref)]
        struct Observed(Mutex<Vec<testing::CounterOutput>>);

        fn write_input(mut inputs: MessageWriter<testing::CounterInput>) {
            inputs.write(testing::CounterInput::Add(5));
        }

        fn read_outputs(
            mut reader: MessageReader<testing::CounterOutput>,
            observed: Res<Observed>,
        ) {
            let observed = observed.into_inner();
            observed.lock().unwrap().extend(reader.read().cloned());
        }

        let mut app = testing::counter_app(StateMachinePlugin::default());
        app.init_resource::<Observed>();
        app.add_systems(
            Update,
            write_input.in_set(StateMachineSystems::collect_inputs::<testing::CounterMachine>()),
        );
        app.add_systems(
            Update,
            read_outputs.in_set(StateMachineSystems::perform_outputs::<
                testing::CounterMachine,
            >()),
        );
        app.update();
        let observed = app.world().resource::<Observed>();
        assert_eq!(*observed.lock().unwrap(), vec![testing::CounterOutput(5)]);
    }

    #[test]
    fn periodic_ticks_once_per_frame() {
        let mut app = testing::counter_app(StateMachinePlugin::default().periodic());
        app.world_mut()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
        app.update();
        app.update();
        app.update();
        assert_eq!(
            testing::drain_messages::<testing::CounterOutput>(&mut app),
            vec![testing::CounterOutput(100), testing::CounterOutput(200)]
        );
    }

    #[test]
    fn periodic_tick_runs_after_same_frame_inputs() {
        let mut app = testing::counter_app(StateMachinePlugin::default().periodic());
        app.world_mut()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
        app.update();
        testing::drain_messages::<testing::CounterOutput>(&mut app);
        app.world_mut()
            .resource_mut::<Messages<testing::CounterInput>>()
            .write(testing::CounterInput::Add(5));
        app.update();
        assert_eq!(
            testing::drain_messages::<testing::CounterOutput>(&mut app),
            vec![testing::CounterOutput(5), testing::CounterOutput(105)]
        );
    }

    #[test]
    fn also_update_in_steps_machine_in_extra_schedule() {
        fn write_input(mut inputs: MessageWriter<testing::CounterInput>) {
            inputs.write(testing::CounterInput::Add(5));
        }

        let mut app = testing::counter_app(StateMachinePlugin::default().also_update_in(Last));
        app.add_systems(
            Last,
            write_input.in_set(StateMachineSystems::collect_inputs::<testing::CounterMachine>()),
        );
        app.update();
        assert_eq!(
            testing::drain_messages::<testing::CounterOutput>(&mut app),
            vec![testing::CounterOutput(5)]
        );
    }

    #[test]
    fn periodic_tick_does_not_run_in_extra_schedule() {
        let mut app = testing::counter_app(
            StateMachinePlugin::default()
                .periodic()
                .also_update_in(Last),
        );
        app.world_mut()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
        app.update();
        testing::drain_messages::<testing::CounterOutput>(&mut app);
        app.update();
        assert_eq!(
            testing::drain_messages::<testing::CounterOutput>(&mut app),
            vec![testing::CounterOutput(100)]
        );
    }

    #[test]
    fn input_from_forwards_events_as_inputs() {
        let mut app =
            testing::counter_app(StateMachinePlugin::default().input_from::<testing::Bump>());
        app.add_message::<testing::Bump>();
        app.world_mut()
            .resource_mut::<Messages<testing::Bump>>()
            .write(testing::Bump(4));
        app.update();
        assert_eq!(
            testing::drain_messages::<testing::CounterOutput>(&mut app),
            vec![testing::CounterOutput(4)]
        );
    }

    #[test]
    fn output_into_forwards_outputs_produced_in_extra_schedule() {
        fn write_input(mut inputs: MessageWriter<testing::CounterInput>) {
            inputs.write(testing::CounterInput::Add(5));
        }

        let mut app = testing::counter_app(
            StateMachinePlugin::default()
                .output_into::<testing::Mirror>()
                .also_update_in(Last),
        );
        app.add_message::<testing::Mirror>();
        app.add_systems(
            Last,
            write_input.in_set(StateMachineSystems::collect_inputs::<testing::CounterMachine>()),
        );
        app.update();
        let mirrored: Vec<u32> = testing::drain_messages::<testing::Mirror>(&mut app)
            .into_iter()
            .map(|mirror| *mirror)
            .collect();
        assert_eq!(mirrored, vec![5]);
    }

    #[test]
    fn probe_update_output_forwarded_once_with_extra_schedule() {
        let mut app = testing::counter_app(
            StateMachinePlugin::default()
                .output_into::<testing::Mirror>()
                .also_update_in(Last),
        );
        app.add_message::<testing::Mirror>();
        app.world_mut()
            .resource_mut::<Messages<testing::CounterInput>>()
            .write(testing::CounterInput::Add(5));
        app.update();
        let mirrored: Vec<u32> = testing::drain_messages::<testing::Mirror>(&mut app)
            .into_iter()
            .map(|mirror| *mirror)
            .collect();
        assert_eq!(mirrored, vec![5]);
    }

    #[test]
    fn two_output_into_adapters_each_forward_every_output() {
        let mut app = testing::counter_app(
            StateMachinePlugin::default()
                .output_into::<testing::Mirror>()
                .output_into::<testing::Echo>(),
        );
        app.add_message::<testing::Mirror>();
        app.add_message::<testing::Echo>();
        app.world_mut()
            .resource_mut::<Messages<testing::CounterInput>>()
            .write(testing::CounterInput::Add(5));
        app.update();
        let mirrored: Vec<u32> = testing::drain_messages::<testing::Mirror>(&mut app)
            .into_iter()
            .map(|mirror| *mirror)
            .collect();
        let echoed: Vec<u32> = testing::drain_messages::<testing::Echo>(&mut app)
            .into_iter()
            .map(|echo| *echo)
            .collect();
        assert_eq!(mirrored, vec![5]);
        assert_eq!(echoed, vec![5]);
    }

    #[test]
    fn probe_input_forwarded_once_with_extra_schedule() {
        let mut app = testing::counter_app(
            StateMachinePlugin::default()
                .input_from::<testing::Bump>()
                .also_update_in(Last),
        );
        app.add_message::<testing::Bump>();
        app.world_mut()
            .resource_mut::<Messages<testing::Bump>>()
            .write(testing::Bump(4));
        app.update();
        assert_eq!(
            testing::drain_messages::<testing::CounterOutput>(&mut app),
            vec![testing::CounterOutput(4)]
        );
    }

    #[test]
    fn output_into_forwards_some_and_drops_none() {
        let mut app =
            testing::counter_app(StateMachinePlugin::default().output_into::<testing::Mirror>());
        app.add_message::<testing::Mirror>();
        app.world_mut()
            .resource_mut::<Messages<testing::CounterInput>>()
            .write(testing::CounterInput::Add(2));
        app.update();
        app.world_mut()
            .resource_mut::<Messages<testing::CounterInput>>()
            .write(testing::CounterInput::Add(5));
        app.update();
        let mirrored: Vec<u32> = testing::drain_messages::<testing::Mirror>(&mut app)
            .into_iter()
            .map(|mirror| *mirror)
            .collect();
        assert_eq!(mirrored, vec![7]);
    }

    #[test]
    fn now_is_refreshed_from_the_machines_clock_each_frame() {
        let mut app = testing::counter_app(StateMachinePlugin::default());
        app.world_mut()
            .insert_resource(Now(testing::FixedMoment(0)));
        app.update();
        assert_eq!(
            **app.world().resource::<Now<testing::FixedMoment>>(),
            testing::FixedMoment(9)
        );
    }

    #[test]
    fn frozen_clock_freezes_now() {
        let mut app = testing::counter_app(StateMachinePlugin::default());
        app.world_mut()
            .insert_resource(Now(testing::FixedMoment(41)));
        app.world_mut().insert_resource(testing::FrozenClock);
        app.update();
        app.update();
        assert_eq!(
            **app.world().resource::<Now<testing::FixedMoment>>(),
            testing::FixedMoment(41)
        );
    }
}
