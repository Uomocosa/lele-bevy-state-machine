use bevy::prelude::{App, Message, Messages};

pub fn drain_messages<M: Message>(app: &mut App) -> Vec<M> {
    app.world_mut()
        .resource_mut::<Messages<M>>()
        .drain()
        .collect()
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{App, Messages};

    use crate::testing;

    use super::drain_messages;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<testing::CounterOutput>();
        app.world_mut()
            .resource_mut::<Messages<testing::CounterOutput>>()
            .write(testing::CounterOutput(1));
        assert_eq!(
            drain_messages::<testing::CounterOutput>(&mut app),
            vec![testing::CounterOutput(1)]
        );
    }
}
