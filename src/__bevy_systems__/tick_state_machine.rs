use bevy::prelude::MessageWriter;

use crate::traits::PeriodicStateMachine;

pub fn write_tick_input<M: PeriodicStateMachine>(mut inputs: MessageWriter<M::Input>) {
    inputs.write(M::tick_input());
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use bevy::prelude::*;

    use crate::traits::{Clock, PeriodicStateMachine, StateMachine};

    #[derive(Message, Debug, PartialEq)]
    enum PulseInput {
        Pulse,
    }

    #[derive(Message)]
    struct PulseOutput;

    #[derive(Clone, Copy)]
    struct PulseNow;

    impl Clock for PulseNow {
        fn now() -> Self {
            Self
        }
    }

    #[derive(Resource)]
    struct Pulser;

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
        fn tick_input() -> Self::Input {
            PulseInput::Pulse
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<PulseInput>>();
        app.add_systems(Update, write_tick_input::<Pulser>);
        app.update();
        let inputs: Vec<PulseInput> = app
            .world_mut()
            .resource_mut::<Messages<PulseInput>>()
            .drain()
            .collect();
        assert_eq!(inputs, vec![PulseInput::Pulse]);
    }
}
