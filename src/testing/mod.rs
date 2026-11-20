#[path = "__app__/mod.rs"]
pub mod app;
#[path = "__basic__/mod.rs"]
mod basic;
#[path = "__fakes__/mod.rs"]
pub mod fakes;
#[path = "__fixtures__/mod.rs"]
pub mod fixtures;
#[path = "__queries__/mod.rs"]
pub mod queries;

pub use app::counter_app::counter_app;
pub use basic::messages::CounterOutput;
pub use basic::messages::DoubleInput;
pub use basic::messages::DoubleOutput;
pub use basic::messages::EmptyInput;
pub use basic::messages::EmptyOutput;
pub use fakes::counter_machine::CounterMachine;
pub use fakes::doubler::Doubler;
pub use fakes::fixed_moment::FixedMoment;
pub use fakes::heartbeat::Heartbeat;
pub use fakes::probe::Probe;
pub use fixtures::bump::Bump;
pub use fixtures::counter_input::CounterInput;
pub use fixtures::echo::Echo;
pub use fixtures::frozen_clock::FrozenClock;
pub use fixtures::mirror::Mirror;
pub use queries::drain_messages::drain_messages;
