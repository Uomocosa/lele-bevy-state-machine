use std::sync::atomic::AtomicU32;

use bevy::prelude::{App, MinimalPlugins};

use crate::StateMachinePlugin;
use crate::testing;

pub fn counter_app(plugin: StateMachinePlugin<testing::CounterMachine>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(plugin);
    app.world_mut()
        .insert_resource(testing::CounterMachine(AtomicU32::new(0)));
    app
}

#[cfg(test)]
mod tests {
    use crate::StateMachinePlugin;
    use crate::testing;

    use super::counter_app;

    #[test]
    fn test_usage() {
        let app = counter_app(StateMachinePlugin::default());
        assert!(app.world().contains_resource::<testing::CounterMachine>());
    }
}
