use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Clone, Copy, Deref)]
pub struct Now<T>(pub T);
