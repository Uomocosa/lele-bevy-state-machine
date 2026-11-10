use bevy::ecs::component::Mutable;
use bevy::prelude::{Message, Resource};

use crate::Clock;

pub trait StateMachine: Resource<Mutability = Mutable> {
    type Input: Message;
    type Output: Message;
    type Now: Clock;
    fn update(&mut self, input: Self::Input, now: Self::Now) -> Vec<Self::Output>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Message;
    use derive_more::Deref;

    use crate::Clock;

    #[derive(Message, Deref)]
    struct DoubleInput(u32);

    #[derive(Message, Debug, Deref, PartialEq)]
    struct DoubleOutput(u32);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestTick;

    impl Clock for TestTick {
        fn now() -> Self {
            Self
        }
    }

    #[derive(Resource)]
    struct Doubler;

    impl StateMachine for Doubler {
        type Input = DoubleInput;
        type Output = DoubleOutput;
        type Now = TestTick;
        fn update(&mut self, input: Self::Input, _now: Self::Now) -> Vec<Self::Output> {
            let DoubleInput(value) = input;
            vec![DoubleOutput(value.wrapping_mul(2))]
        }
    }

    #[test]
    fn test_usage() {
        let mut machine = Doubler;
        assert_eq!(
            machine.update(DoubleInput(21), TestTick),
            vec![DoubleOutput(42)]
        );
    }
}
