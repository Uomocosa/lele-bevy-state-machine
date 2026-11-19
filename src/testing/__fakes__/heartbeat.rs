use std::time::Duration;

use bevy::prelude::Resource;

use crate::testing;
use crate::traits::{PeriodicStateMachine, StateMachine};

#[derive(Resource)]
pub struct Heartbeat;

#[rustfmt::skip]
impl StateMachine for Heartbeat {
    type Input = testing::EmptyInput;
    type Output = testing::EmptyOutput;
    type Now = testing::FixedMoment;

    fn update(&mut self, _input: Self::Input, _now: Self::Now) -> Vec<Self::Output> { Vec::new() }
}

#[rustfmt::skip]
impl PeriodicStateMachine for Heartbeat {
    const TICK_INTERVAL: Duration = Duration::from_millis(50);

    fn tick(&mut self, _now: Self::Now) -> Vec<Self::Output> { vec![testing::EmptyOutput] }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::testing;
    use crate::traits::{Clock, PeriodicStateMachine};

    use super::Heartbeat;

    #[test]
    fn test_usage() {
        assert_eq!(Heartbeat::TICK_INTERVAL, Duration::from_millis(50));
        assert_eq!(Heartbeat.tick(testing::FixedMoment::now()).len(), 1);
    }
}
