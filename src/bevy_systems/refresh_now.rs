use bevy::prelude::ResMut;

use crate::basic;
use crate::clock::Clock;

pub fn refresh_now<T: Clock>(mut now: ResMut<basic::resources::Now<T>>) {
    *now = basic::resources::Now(T::now());
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use derive_more::Deref;

    use crate::basic;
    use crate::clock::Clock;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deref)]
    struct FixedNow(u64);

    impl Clock for FixedNow {
        fn now() -> Self {
            Self(77)
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(basic::resources::Now(FixedNow(0)));
        app.add_systems(Update, refresh_now::<FixedNow>);
        app.update();
        assert_eq!(
            **app.world().resource::<basic::resources::Now<FixedNow>>(),
            FixedNow(77)
        );
    }
}
