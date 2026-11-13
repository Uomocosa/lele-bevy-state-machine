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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    use bevy::prelude::*;
    use derive_more::Deref;

    use crate::traits::{Clock, PeriodicStateMachine, StateMachine};

    #[derive(Message, Debug, PartialEq)]
    struct PulseInput;

    #[derive(Message, Debug, Deref, PartialEq)]
    struct PulseOutput(u32);

    #[derive(Clone, Copy)]
    struct PulseNow;

    impl Clock for PulseNow {
        fn now() -> Self {
            Self
        }
    }

    #[derive(Resource, Deref)]
    struct Pulser(AtomicU32);

    impl StateMachine for Pulser {
        type Input = PulseInput;
        type Output = PulseOutput;
        type Now = PulseNow;
        fn update(&mut self, _input: Self::Input, _now: Self::Now) -> Vec<Self::Output> {
            Vec::new()
        }
    }

    impl PeriodicStateMachine for Pulser {
        const TICK_INTERVAL: Duration = Duration::from_millis(10);
        fn tick(&mut self, _now: Self::Now) -> Vec<Self::Output> {
            vec![PulseOutput(self.fetch_add(1, Ordering::SeqCst))]
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Now(PulseNow));
        app.insert_resource(Pulser(AtomicU32::new(0)));
        app.add_message::<PulseOutput>();
        app.add_systems(Update, tick_state_machine::<Pulser>);
        app.update();
        let outputs: Vec<PulseOutput> = app
            .world_mut()
            .resource_mut::<Messages<PulseOutput>>()
            .drain()
            .collect();
        assert_eq!(outputs, vec![PulseOutput(0)]);
    }
}
