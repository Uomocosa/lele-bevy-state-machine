use bevy::prelude::{Message, MessageWriter, Messages, Res, ResMut};

use crate::forward_cursor::ForwardCursor;
use crate::traits::StateMachine;

pub fn forward_input<M: StateMachine, E: Message>(
    mut forwarded: ResMut<ForwardCursor<M, E>>,
    events: Res<Messages<E>>,
    mut inputs: MessageWriter<M::Input>,
) where
    for<'e> M::Input: From<&'e E>,
{
    let events = events.into_inner();
    inputs.write_batch(forwarded.cursor.read(events).map(M::Input::from));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use derive_more::Deref;

    use crate::traits::{Clock, StateMachine};

    #[derive(Message, Deref)]
    struct Press(u32);

    #[derive(Message, Debug, Deref, PartialEq)]
    struct Pressed(u32);

    #[derive(Clone, Copy)]
    struct PressNow;

    impl Clock for PressNow {
        fn now() -> Self {
            Self
        }
    }

    #[derive(Resource)]
    struct Keyboard;

    impl StateMachine for Keyboard {
        type Input = Pressed;
        type Output = Pressed;
        type Now = PressNow;
        fn update(&mut self, _input: Self::Input, _now: Self::Now) -> Vec<Self::Output> {
            Vec::new()
        }
    }

    impl From<&Press> for Pressed {
        fn from(press: &Press) -> Self {
            Self(**press)
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<Press>();
        app.init_resource::<Messages<Pressed>>();
        app.init_resource::<ForwardCursor<Keyboard, Press>>();
        app.add_systems(Update, forward_input::<Keyboard, Press>);
        app.world_mut()
            .resource_mut::<Messages<Press>>()
            .write(Press(7));
        app.update();
        let inputs: Vec<Pressed> = app
            .world_mut()
            .resource_mut::<Messages<Pressed>>()
            .drain()
            .collect();
        assert_eq!(inputs, vec![Pressed(7)]);
    }
}
