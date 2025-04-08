use crate::gameplay::player::PLAYER_SIZE;
use bevy::prelude::*;

pub const WORLD_SIZE: f32 = 1000.0;
pub const WORLD_BOUNDARY_VECTOR: Vec3 = Vec3::new(
    0.5 * WORLD_SIZE - 0.5 * PLAYER_SIZE,
    0.5 * WORLD_SIZE - 0.5 * PLAYER_SIZE,
    0.0,
);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum State {
    #[default]
    Playing,
    GameOver,
}
