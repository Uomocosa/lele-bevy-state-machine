use bevy::ecs::component::Mutable;
use bevy::prelude::{Message, Resource};

use crate::traits::Clock;

pub trait StateMachine: Resource<Mutability = Mutable> {
    type Input: Message;
    type Output: Message;
    type Now: Clock;
    fn update(&mut self, input: Self::Input, now: Self::Now) -> Vec<Self::Output>;
}

#[cfg(test)]
mod tests {
    use crate::testing;
    use crate::traits::{Clock, StateMachine};

    #[test]
    fn test_usage() {
        let mut machine = testing::Doubler;
        assert_eq!(
            machine.update(testing::DoubleInput(21), testing::FixedMoment::now()),
            vec![testing::DoubleOutput(42)]
        );
    }
}
