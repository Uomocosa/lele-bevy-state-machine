use bevy::prelude::{Message, MessageReader, MessageWriter};

use crate::traits::StateMachine;

pub fn forward_output<M: StateMachine, T: Message>(
    mut outputs: MessageReader<M::Output>,
    mut forwarded: MessageWriter<T>,
) where
    for<'o> Option<T>: From<&'o M::Output>,
{
    forwarded.write_batch(outputs.read().filter_map(Option::<T>::from));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use derive_more::Deref;

    use crate::traits::{Clock, StateMachine};

    #[derive(Message, Debug, Deref, PartialEq)]
    struct Count(u32);

    #[derive(Message, Deref)]
    struct Total(u32);

    #[derive(Clone, Copy)]
    struct CountNow;

    impl Clock for CountNow {
        fn now() -> Self {
            Self
        }
    }

    #[derive(Resource)]
    struct Counter;

    impl StateMachine for Counter {
        type Input = Count;
        type Output = Count;
        type Now = CountNow;
        fn update(&mut self, _input: Self::Input, _now: Self::Now) -> Vec<Self::Output> {
            Vec::new()
        }
    }

    impl From<&Count> for Option<Total> {
        fn from(count: &Count) -> Self {
            (**count > 1).then_some(Total(**count))
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<Count>();
        app.add_message::<Total>();
        app.add_systems(Update, forward_output::<Counter, Total>);
        app.world_mut()
            .resource_mut::<Messages<Count>>()
            .write_batch([Count(1), Count(4)]);
        app.update();
        let totals: Vec<u32> = app
            .world_mut()
            .resource_mut::<Messages<Total>>()
            .drain()
            .map(|total| *total)
            .collect();
        assert_eq!(totals, vec![4]);
    }
}
