#[path = "__basic__/mod.rs"]
mod basic;
#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;
#[path = "../__methods__/mod.rs"]
mod methods;
mod state_machine_plugin;
mod state_machine_systems;
#[cfg(feature = "testing")]
pub mod testing;
#[path = "__traits__/mod.rs"]
pub mod traits;

pub use basic::enums::StateMachineStage;
pub use basic::resources::Now;
pub use state_machine_plugin::StateMachinePlugin;
pub use state_machine_systems::StateMachineSystems;

// no test_usage necessary
