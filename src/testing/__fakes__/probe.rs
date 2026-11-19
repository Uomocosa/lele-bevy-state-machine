use bevy::prelude::Resource;

use crate::testing;
use crate::traits::StateMachine;

#[derive(Resource)]
pub struct Probe;

#[rustfmt::skip]
impl StateMachine for Probe {
    type Input = testing::EmptyInput;
    type Output = testing::EmptyOutput;
    type Now = testing::FixedMoment;

    fn update(&mut self, _input: Self::Input, _now: Self::Now) -> Vec<Self::Output> { Vec::new() }
}

#[cfg(test)]
mod tests {
    use crate::testing;
    use crate::traits::{Clock, StateMachine};

    use super::Probe;

    #[test]
    fn test_usage() {
        assert!(
            Probe
                .update(testing::EmptyInput, testing::FixedMoment::now())
                .is_empty()
        );
    }
}
