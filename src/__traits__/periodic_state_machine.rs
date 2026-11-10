use std::time::Duration;

use crate::StateMachine;

pub trait PeriodicStateMachine: StateMachine {
    const TICK_INTERVAL: Duration;
    fn tick_input() -> Self::Input;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{Message, Resource};

    use crate::Clock;
    use crate::StateMachine;

    #[derive(Message, Debug, PartialEq)]
    enum HeartbeatInput {
        Beat,
    }

    #[derive(Message)]
    struct HeartbeatOutput;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestTick;

    impl Clock for TestTick {
        fn now() -> Self {
            Self
        }
    }

    #[derive(Resource)]
    struct Heartbeat;

    impl StateMachine for Heartbeat {
        type Input = HeartbeatInput;
        type Output = HeartbeatOutput;
        type Now = TestTick;
        fn update(&mut self, _input: Self::Input, _now: Self::Now) -> Vec<Self::Output> {
            Vec::new()
        }
    }

    impl PeriodicStateMachine for Heartbeat {
        const TICK_INTERVAL: Duration = Duration::from_millis(50);
        fn tick_input() -> Self::Input {
            HeartbeatInput::Beat
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(Heartbeat::TICK_INTERVAL, Duration::from_millis(50));
        assert_eq!(Heartbeat::tick_input(), HeartbeatInput::Beat);
    }
}
