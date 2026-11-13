use std::marker::PhantomData;

use atomic_delegate_macros::atomic_delegates;
use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use bevy::prelude::{App, IntoScheduleConfigs, Message, Plugin, Update};
use bevy::time::common_conditions::on_timer;

use crate::bevy_systems;
use crate::forward_cursor::ForwardCursor;
use crate::state_machine_systems::StateMachineSystems;
use crate::traits::{PeriodicStateMachine, StateMachine};

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

#[rustfmt::skip]
impl<M: StateMachine> Plugin for StateMachinePlugin<M> {
    fn build(&self, app: &mut App) { self.build_plugin(app); }
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
            app.init_resource::<ForwardCursor<M, M::Output>>();
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
    pub(crate) fn build_plugin(&self, app: &mut App) {}

    pub(crate) fn configure_update_schedule<Machine: StateMachine>(
        app: &mut App,
        schedule: InternedScheduleLabel,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    use super::*;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use derive_more::Deref;

    use crate::Now;
    #[cfg(feature = "testing")]
    use crate::testing;
    use crate::traits::Clock;

    #[derive(Resource, Deref)]
    struct CounterMachine(AtomicU32);

    #[derive(Debug, Clone, PartialEq, Message)]
    enum CounterInput {
        Add(u32),
    }

    #[derive(Debug, Clone, Deref, PartialEq, Message)]
    struct CounterOutput(u32);

    #[derive(Debug, Clone, Deref, PartialEq, Message)]
    struct Bump(u32);

    #[derive(Debug, Clone, Deref, PartialEq, Message)]
    struct Mirror(u32);

    #[derive(Debug, Clone, Copy, Deref, PartialEq, Eq)]
    struct TestNow(u64);

    impl Clock for TestNow {
        fn now() -> Self {
            Self(11)
        }
    }

    impl StateMachine for CounterMachine {
        type Input = CounterInput;
        type Output = CounterOutput;
        type Now = TestNow;
        fn update(&mut self, input: Self::Input, _now: Self::Now) -> Vec<Self::Output> {
            let CounterInput::Add(value) = input;
            self.store(
                self.load(Ordering::SeqCst).wrapping_add(value),
                Ordering::SeqCst,
            );
            vec![CounterOutput(self.load(Ordering::SeqCst))]
        }
    }

    impl PeriodicStateMachine for CounterMachine {
        const TICK_INTERVAL: Duration = Duration::from_millis(1);
        fn tick(&mut self, _now: Self::Now) -> Vec<Self::Output> {
            self.fetch_add(100, Ordering::SeqCst);
            vec![CounterOutput(self.load(Ordering::SeqCst))]
        }
    }

    impl From<&Bump> for CounterInput {
        fn from(bump: &Bump) -> Self {
            Self::Add(**bump)
        }
    }

    impl From<&CounterOutput> for Option<Mirror> {
        fn from(output: &CounterOutput) -> Self {
            (**output > 3).then_some(Mirror(**output))
        }
    }

    fn counter_app(plugin: StateMachinePlugin<CounterMachine>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin);
        app.world_mut()
            .insert_resource(CounterMachine(AtomicU32::new(0)));
        app
    }

    fn drain_outputs(app: &mut App) -> Vec<CounterOutput> {
        app.world_mut()
            .resource_mut::<Messages<CounterOutput>>()
            .drain()
            .collect()
    }

    #[test]
    fn test_usage() {
        let mut app = counter_app(StateMachinePlugin::default());
        app.world_mut()
            .resource_mut::<Messages<CounterInput>>()
            .write(CounterInput::Add(2));
        app.update();
        assert_eq!(drain_outputs(&mut app), vec![CounterOutput(2)]);
    }

    #[test]
    fn inputs_written_before_machine_insert_are_not_lost() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StateMachinePlugin::<CounterMachine>::default());
        app.world_mut()
            .resource_mut::<Messages<CounterInput>>()
            .write(CounterInput::Add(3));
        app.update();
        app.world_mut()
            .insert_resource(CounterMachine(AtomicU32::new(0)));
        app.update();
        assert_eq!(drain_outputs(&mut app), vec![CounterOutput(3)]);
    }

    #[test]
    fn outputs_readable_in_perform_outputs_same_frame() {
        #[derive(Resource, Default, Deref)]
        struct Observed(Mutex<Vec<CounterOutput>>);

        fn write_input(mut inputs: MessageWriter<CounterInput>) {
            inputs.write(CounterInput::Add(5));
        }

        fn read_outputs(mut reader: MessageReader<CounterOutput>, observed: Res<Observed>) {
            let observed = observed.into_inner();
            observed.lock().unwrap().extend(reader.read().cloned());
        }

        let mut app = counter_app(StateMachinePlugin::default());
        app.init_resource::<Observed>();
        app.add_systems(
            Update,
            write_input.in_set(StateMachineSystems::collect_inputs::<CounterMachine>()),
        );
        app.add_systems(
            Update,
            read_outputs.in_set(StateMachineSystems::perform_outputs::<CounterMachine>()),
        );
        app.update();
        let observed = app.world().resource::<Observed>();
        assert_eq!(*observed.lock().unwrap(), vec![CounterOutput(5)]);
    }

    #[test]
    fn periodic_ticks_once_per_frame() {
        let mut app = counter_app(StateMachinePlugin::default().periodic());
        app.world_mut()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
        app.update();
        app.update();
        app.update();
        assert_eq!(
            drain_outputs(&mut app),
            vec![CounterOutput(100), CounterOutput(200)]
        );
    }

    #[test]
    fn periodic_tick_runs_after_same_frame_inputs() {
        let mut app = counter_app(StateMachinePlugin::default().periodic());
        app.world_mut()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
        app.update();
        drain_outputs(&mut app);
        app.world_mut()
            .resource_mut::<Messages<CounterInput>>()
            .write(CounterInput::Add(5));
        app.update();
        assert_eq!(
            drain_outputs(&mut app),
            vec![CounterOutput(5), CounterOutput(105)]
        );
    }

    #[test]
    fn also_update_in_steps_machine_in_extra_schedule() {
        fn write_input(mut inputs: MessageWriter<CounterInput>) {
            inputs.write(CounterInput::Add(5));
        }

        let mut app = counter_app(StateMachinePlugin::default().also_update_in(Last));
        app.add_systems(
            Last,
            write_input.in_set(StateMachineSystems::collect_inputs::<CounterMachine>()),
        );
        app.update();
        assert_eq!(drain_outputs(&mut app), vec![CounterOutput(5)]);
    }

    #[test]
    fn periodic_tick_does_not_run_in_extra_schedule() {
        let mut app = counter_app(
            StateMachinePlugin::default()
                .periodic()
                .also_update_in(Last),
        );
        app.world_mut()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
        app.update();
        drain_outputs(&mut app);
        app.update();
        assert_eq!(drain_outputs(&mut app), vec![CounterOutput(100)]);
    }

    #[test]
    fn input_from_forwards_events_as_inputs() {
        let mut app = counter_app(StateMachinePlugin::default().input_from::<Bump>());
        app.add_message::<Bump>();
        app.world_mut()
            .resource_mut::<Messages<Bump>>()
            .write(Bump(4));
        app.update();
        assert_eq!(drain_outputs(&mut app), vec![CounterOutput(4)]);
    }

    #[test]
    fn output_into_forwards_outputs_produced_in_extra_schedule() {
        fn write_input(mut inputs: MessageWriter<CounterInput>) {
            inputs.write(CounterInput::Add(5));
        }

        let mut app = counter_app(
            StateMachinePlugin::default()
                .output_into::<Mirror>()
                .also_update_in(Last),
        );
        app.add_message::<Mirror>();
        app.add_systems(
            Last,
            write_input.in_set(StateMachineSystems::collect_inputs::<CounterMachine>()),
        );
        app.update();
        let mirrored: Vec<u32> = app
            .world_mut()
            .resource_mut::<Messages<Mirror>>()
            .drain()
            .map(|mirror| *mirror)
            .collect();
        assert_eq!(mirrored, vec![5]);
    }

    #[test]
    fn probe_update_output_forwarded_once_with_extra_schedule() {
        let mut app = counter_app(
            StateMachinePlugin::default()
                .output_into::<Mirror>()
                .also_update_in(Last),
        );
        app.add_message::<Mirror>();
        app.world_mut()
            .resource_mut::<Messages<CounterInput>>()
            .write(CounterInput::Add(5));
        app.update();
        let mirrored: Vec<u32> = app
            .world_mut()
            .resource_mut::<Messages<Mirror>>()
            .drain()
            .map(|mirror| *mirror)
            .collect();
        assert_eq!(mirrored, vec![5]);
    }

    #[test]
    fn probe_input_forwarded_once_with_extra_schedule() {
        let mut app = counter_app(
            StateMachinePlugin::default()
                .input_from::<Bump>()
                .also_update_in(Last),
        );
        app.add_message::<Bump>();
        app.world_mut()
            .resource_mut::<Messages<Bump>>()
            .write(Bump(4));
        app.update();
        assert_eq!(drain_outputs(&mut app), vec![CounterOutput(4)]);
    }

    #[test]
    fn output_into_forwards_some_and_drops_none() {
        let mut app = counter_app(StateMachinePlugin::default().output_into::<Mirror>());
        app.add_message::<Mirror>();
        app.world_mut()
            .resource_mut::<Messages<CounterInput>>()
            .write(CounterInput::Add(2));
        app.update();
        app.world_mut()
            .resource_mut::<Messages<CounterInput>>()
            .write(CounterInput::Add(5));
        app.update();
        let mirrored: Vec<u32> = app
            .world_mut()
            .resource_mut::<Messages<Mirror>>()
            .drain()
            .map(|mirror| *mirror)
            .collect();
        assert_eq!(mirrored, vec![7]);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn frozen_clock_freezes_now() {
        let mut app = counter_app(StateMachinePlugin::default());
        app.world_mut().insert_resource(Now(TestNow(41)));
        app.world_mut().insert_resource(testing::FrozenClock);
        app.update();
        app.update();
        assert_eq!(**app.world().resource::<Now<TestNow>>(), TestNow(41));
    }
}
