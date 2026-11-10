use std::marker::PhantomData;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::{App, IntoScheduleConfigs, Plugin, Update};
use bevy::time::common_conditions::on_timer;

use crate::PeriodicStateMachine;
use crate::StateMachine;
use crate::bevy_systems;
use crate::state_machine_systems::StateMachineSystems;

pub struct StateMachinePlugin<M> {
    pub(crate) add_tick_system: Option<fn(&mut App)>,
    pub(crate) is_stepped_on_exit: bool,
    marker: PhantomData<fn() -> M>,
}

impl<M: StateMachine> Default for StateMachinePlugin<M> {
    fn default() -> Self {
        Self {
            add_tick_system: None,
            is_stepped_on_exit: false,
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
    pub const fn steps_on_exit(mut self) -> Self {
        self.is_stepped_on_exit = true;
        self
    }
}

impl<M: PeriodicStateMachine> StateMachinePlugin<M> {
    #[must_use]
    pub fn periodic(mut self) -> Self {
        self.add_tick_system = Some(|app: &mut App| {
            app.add_systems(
                Update,
                bevy_systems::write_tick_input::<M>
                    .in_set(StateMachineSystems::collect_inputs::<M>())
                    .run_if(on_timer(M::TICK_INTERVAL)),
            );
        });
        self
    }
}

#[atomic_delegates]
impl<M: StateMachine> StateMachinePlugin<M> {
    pub(crate) fn build_plugin(&self, app: &mut App) {}
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

    use crate::Clock;
    #[cfg(feature = "testing")]
    use crate::basic;
    #[cfg(feature = "testing")]
    use crate::testing;

    #[derive(Resource, Deref)]
    struct CounterMachine(AtomicU32);

    #[derive(Debug, Clone, PartialEq, Message)]
    enum CounterInput {
        Add(u32),
        Tick,
    }

    #[derive(Debug, Clone, Deref, PartialEq, Message)]
    struct CounterOutput(u32);

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
            if let CounterInput::Add(value) = input {
                self.store(
                    self.load(Ordering::SeqCst).wrapping_add(value),
                    Ordering::SeqCst,
                );
            }
            vec![CounterOutput(self.load(Ordering::SeqCst))]
        }
    }

    impl PeriodicStateMachine for CounterMachine {
        const TICK_INTERVAL: Duration = Duration::from_millis(1);
        fn tick_input() -> Self::Input {
            CounterInput::Tick
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
    fn periodic_produces_tick_inputs() {
        let mut app = counter_app(StateMachinePlugin::default().periodic());
        app.world_mut()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
        app.world_mut()
            .resource_mut::<CounterMachine>()
            .store(7, Ordering::SeqCst);
        app.update();
        app.update();
        let outputs = drain_outputs(&mut app);
        assert!(matches!(outputs.as_slice(), [_, ..]));
        assert!(outputs.iter().all(|output| *output == CounterOutput(7)));
    }

    #[test]
    fn steps_on_exit_steps_last_frame_input() {
        fn write_input(mut inputs: MessageWriter<CounterInput>) {
            inputs.write(CounterInput::Add(5));
        }

        let mut app = counter_app(StateMachinePlugin::default().steps_on_exit());
        app.add_systems(
            Last,
            write_input.in_set(StateMachineSystems::collect_inputs::<CounterMachine>()),
        );
        app.update();
        assert_eq!(drain_outputs(&mut app), vec![CounterOutput(5)]);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn frozen_clock_freezes_now() {
        let mut app = counter_app(StateMachinePlugin::default());
        app.world_mut()
            .insert_resource(basic::resources::Now(TestNow(41)));
        app.world_mut().insert_resource(testing::FrozenClock);
        app.update();
        app.update();
        assert_eq!(
            **app.world().resource::<basic::resources::Now<TestNow>>(),
            TestNow(41)
        );
    }
}
