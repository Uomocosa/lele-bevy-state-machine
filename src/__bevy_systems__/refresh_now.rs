use bevy::prelude::ResMut;

use crate::Now;
use crate::traits::Clock;

pub fn refresh_now<T: Clock>(mut now: ResMut<Now<T>>) {
    *now = Now(T::now());
}
