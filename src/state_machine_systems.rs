use std::any::TypeId;

use bevy::prelude::SystemSet;

use crate::basic;
use crate::state_machine::StateMachine;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateMachineSystems {
    pub machine: TypeId,
    pub stage: basic::enums::StateMachineStage,
}

impl StateMachineSystems {
    #[must_use]
    pub const fn collect_inputs<M: StateMachine>() -> Self {
        Self {
            machine: TypeId::of::<M>(),
            stage: basic::enums::StateMachineStage::CollectInputs,
        }
    }

    #[must_use]
    pub const fn step<M: StateMachine>() -> Self {
        Self {
            machine: TypeId::of::<M>(),
            stage: basic::enums::StateMachineStage::Step,
        }
    }

    #[must_use]
    pub const fn perform_outputs<M: StateMachine>() -> Self {
        Self {
            machine: TypeId::of::<M>(),
            stage: basic::enums::StateMachineStage::PerformOutputs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{Message, Resource};

    use crate::clock::Clock;
    use crate::state_machine::StateMachine;

    #[derive(Message)]
    struct ProbeInput;

    #[derive(Message)]
    struct ProbeOutput;

    #[derive(Clone, Copy)]
    struct ProbeNow;

    impl Clock for ProbeNow {
        fn now() -> Self {
            Self
        }
    }

    #[derive(Resource)]
    struct Probe;

    impl StateMachine for Probe {
        type Input = ProbeInput;
        type Output = ProbeOutput;
        type Now = ProbeNow;
        fn update(&mut self, _input: Self::Input, _now: Self::Now) -> Vec<Self::Output> {
            Vec::new()
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            StateMachineSystems::collect_inputs::<Probe>().stage,
            basic::enums::StateMachineStage::CollectInputs
        );
        assert_eq!(
            StateMachineSystems::step::<Probe>().stage,
            basic::enums::StateMachineStage::Step
        );
        assert_eq!(
            StateMachineSystems::perform_outputs::<Probe>().stage,
            basic::enums::StateMachineStage::PerformOutputs
        );
        assert_eq!(
            StateMachineSystems::step::<Probe>().machine,
            TypeId::of::<Probe>()
        );
    }
}
