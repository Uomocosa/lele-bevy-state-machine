use bevy::prelude::Resource;

use crate::testing;
use crate::traits::StateMachine;

#[derive(Resource)]
pub struct Doubler;

impl StateMachine for Doubler {
    type Input = testing::DoubleInput;
    type Output = testing::DoubleOutput;
    type Now = testing::FixedMoment;

    fn update(&mut self, input: Self::Input, _now: Self::Now) -> Vec<Self::Output> {
        let testing::DoubleInput(value) = input;
        vec![testing::DoubleOutput(value.wrapping_mul(2))]
    }
}

#[cfg(test)]
mod tests {
    use crate::testing;
    use crate::traits::{Clock, StateMachine};

    use super::Doubler;

    #[test]
    fn test_usage() {
        let mut machine = Doubler;
        assert_eq!(
            machine.update(testing::DoubleInput(21), testing::FixedMoment::now()),
            vec![testing::DoubleOutput(42)]
        );
    }
}
