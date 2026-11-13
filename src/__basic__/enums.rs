#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateMachineStage {
    CollectInputs,
    ApplyInputs,
    PerformOutputs,
}
