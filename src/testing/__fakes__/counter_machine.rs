use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use bevy::prelude::Resource;
use derive_more::Deref;

use crate::testing;
use crate::traits::{PeriodicStateMachine, StateMachine};

#[derive(Resource, Deref)]
pub struct CounterMachine(pub AtomicU32);

impl StateMachine for CounterMachine {
    type Input = testing::CounterInput;
    type Output = testing::CounterOutput;
    type Now = testing::FixedMoment;

    fn update(&mut self, input: Self::Input, _now: Self::Now) -> Vec<Self::Output> {
        let testing::CounterInput::Add(value) = input;
        self.store(
            self.load(Ordering::SeqCst).wrapping_add(value),
            Ordering::SeqCst,
        );
        vec![testing::CounterOutput(self.load(Ordering::SeqCst))]
    }
}

impl PeriodicStateMachine for CounterMachine {
    const TICK_INTERVAL: Duration = Duration::from_millis(1);

    fn tick(&mut self, _now: Self::Now) -> Vec<Self::Output> {
        self.fetch_add(100, Ordering::SeqCst);
        vec![testing::CounterOutput(self.load(Ordering::SeqCst))]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU32;

    use crate::testing;
    use crate::traits::{Clock, PeriodicStateMachine, StateMachine};

    use super::CounterMachine;

    #[test]
    fn test_usage() {
        let mut machine = CounterMachine(AtomicU32::new(0));
        assert_eq!(
            machine.update(testing::CounterInput::Add(2), testing::FixedMoment::now()),
            vec![testing::CounterOutput(2)]
        );
        assert_eq!(
            machine.tick(testing::FixedMoment::now()),
            vec![testing::CounterOutput(102)]
        );
    }
}
