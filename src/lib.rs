#[path = "__basic__/mod.rs"]
pub mod basic;
pub mod bevy_systems;
mod clock;
#[path = "../methods/mod.rs"]
mod methods;
mod periodic_state_machine;
mod state_machine;
mod state_machine_plugin;
mod state_machine_systems;
#[cfg(feature = "testing")]
pub mod testing;

pub use basic::enums::StateMachineStage;
pub use basic::resources::Now;
pub use clock::Clock;
pub use periodic_state_machine::PeriodicStateMachine;
pub use state_machine::StateMachine;
pub use state_machine_plugin::StateMachinePlugin;
pub use state_machine_systems::StateMachineSystems;

// no test_usage necessary
