use bevy::prelude::ResMut;

use crate::Now;
use crate::traits::Clock;

pub fn refresh_now<T: Clock>(mut now: ResMut<Now<T>>) {
    *now = Now(T::now());
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use derive_more::Deref;

    use crate::Now;
    use crate::traits::Clock;

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
        app.insert_resource(Now(FixedNow(0)));
        app.add_systems(Update, refresh_now::<FixedNow>);
        app.update();
        assert_eq!(**app.world().resource::<Now<FixedNow>>(), FixedNow(77));
    }
}
