use std::time::Duration;

use crate::traits::StateMachine;

pub trait PeriodicStateMachine: StateMachine {
    const TICK_INTERVAL: Duration;
    fn tick(&mut self, now: Self::Now) -> Vec<Self::Output>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{Message, Resource};

    use crate::traits::{Clock, StateMachine};

    #[derive(Message, Debug, PartialEq)]
    struct HeartbeatInput;

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
        fn tick(&mut self, _now: Self::Now) -> Vec<Self::Output> {
            vec![HeartbeatOutput]
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(Heartbeat::TICK_INTERVAL, Duration::from_millis(50));
        assert_eq!(Heartbeat.tick(TestTick).len(), 1);
    }
}
