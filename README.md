# lele-bevy-state-machine

Drives pure sans-IO state machines from Bevy 0.20. The machine owns its state,
receives inputs as Bevy messages, and returns outputs as Bevy messages; the
plugin wires the schedules around it.

```rust
use bevy::prelude::*;
use lele_bevy_state_machine::{StateMachine, StateMachinePlugin};

fn main() {
    App::new().add_plugins((MinimalPlugins, StateMachinePlugin::<MyMachine>::default()));
}
```
