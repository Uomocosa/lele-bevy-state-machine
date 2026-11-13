use bevy::prelude::{MessageWriter, Messages, Res, ResMut};

use crate::Now;
use crate::traits::StateMachine;

pub fn update_state_machine<M: StateMachine>(
    mut machine: ResMut<M>,
    mut inputs: ResMut<Messages<M::Input>>,
    mut outputs: MessageWriter<M::Output>,
    now: Res<Now<M::Now>>,
) {
    let now = now.into_inner();
    for input in inputs.drain() {
        outputs.write_batch(machine.update(input, **now));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;
    use bevy::prelude::*;
    use derive_more::Deref;

    use crate::Now;
    use crate::traits::{Clock, StateMachine};

    #[derive(Message, Deref)]
    struct BumpInput(u32);

    #[derive(Message, Debug, Deref, PartialEq)]
    struct BumpOutput(u32);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct BumpNow;

    impl Clock for BumpNow {
        fn now() -> Self {
            Self
        }
    }

    #[derive(Resource, Deref)]
    struct Bumper(AtomicU32);

    impl StateMachine for Bumper {
        type Input = BumpInput;
        type Output = BumpOutput;
        type Now = BumpNow;
        fn update(&mut self, input: Self::Input, _now: Self::Now) -> Vec<Self::Output> {
            let BumpInput(value) = input;
            self.store(
                self.load(Ordering::SeqCst).wrapping_add(value),
                Ordering::SeqCst,
            );
            vec![BumpOutput(self.load(Ordering::SeqCst))]
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<BumpInput>>();
        app.add_message::<BumpOutput>();
        app.insert_resource(Now(BumpNow));
        app.insert_resource(Bumper(AtomicU32::new(1)));
        app.add_systems(Update, update_state_machine::<Bumper>);
        app.world_mut()
            .resource_mut::<Messages<BumpInput>>()
            .write(BumpInput(41));
        app.update();
        let outputs: Vec<BumpOutput> = app
            .world_mut()
            .resource_mut::<Messages<BumpOutput>>()
            .drain()
            .collect();
        assert_eq!(outputs, vec![BumpOutput(42)]);
    }
}
