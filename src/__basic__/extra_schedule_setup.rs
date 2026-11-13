use bevy::prelude::App;

pub type ExtraScheduleSetup = Box<dyn Fn(&mut App) + Send + Sync>;
