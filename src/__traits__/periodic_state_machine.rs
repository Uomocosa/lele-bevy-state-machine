use std::time::Duration;

use crate::traits::StateMachine;

pub trait PeriodicStateMachine: StateMachine {
    const TICK_INTERVAL: Duration;
    fn tick(&mut self, now: Self::Now) -> Vec<Self::Output>;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::testing;
    use crate::traits::{Clock, PeriodicStateMachine};

    #[test]
    fn test_usage() {
        assert_eq!(testing::Heartbeat::TICK_INTERVAL, Duration::from_millis(50));
        assert_eq!(
            testing::Heartbeat.tick(testing::FixedMoment::now()).len(),
            1
        );
    }
}
