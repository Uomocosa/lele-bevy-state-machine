use std::any::TypeId;

use bevy::prelude::SystemSet;

use crate::StateMachineStage;
use crate::traits::StateMachine;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateMachineSystems {
    pub machine: TypeId,
    pub stage: StateMachineStage,
}

impl StateMachineSystems {
    #[must_use]
    pub const fn collect_inputs<M: StateMachine>() -> Self {
        Self {
            machine: TypeId::of::<M>(),
            stage: StateMachineStage::CollectInputs,
        }
    }

    #[must_use]
    pub const fn apply_inputs<M: StateMachine>() -> Self {
        Self {
            machine: TypeId::of::<M>(),
            stage: StateMachineStage::ApplyInputs,
        }
    }

    #[must_use]
    pub const fn perform_outputs<M: StateMachine>() -> Self {
        Self {
            machine: TypeId::of::<M>(),
            stage: StateMachineStage::PerformOutputs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing;

    #[test]
    fn test_usage() {
        assert_eq!(
            StateMachineSystems::collect_inputs::<testing::Probe>().stage,
            StateMachineStage::CollectInputs
        );
        assert_eq!(
            StateMachineSystems::apply_inputs::<testing::Probe>().stage,
            StateMachineStage::ApplyInputs
        );
        assert_eq!(
            StateMachineSystems::perform_outputs::<testing::Probe>().stage,
            StateMachineStage::PerformOutputs
        );
        assert_eq!(
            StateMachineSystems::apply_inputs::<testing::Probe>().machine,
            TypeId::of::<testing::Probe>()
        );
    }
}
