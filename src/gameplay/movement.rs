use bevy::prelude::*;
use crate::world::State;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedUpdate, apply_velocity.run_if(in_state(State::Playing)));
    }
}

#[derive(Component, Default)]
pub struct Velocity(pub Vec3);

impl Velocity {
    pub fn new(val: Vec3) -> Self {
        Velocity(val)
    }
}

fn apply_velocity(
    mut query: Query<(&Velocity, &mut Transform)>,
    time: Res<Time>,
) {
    for result in query.iter_mut() {
        let (velocity, mut transform) = result;
        transform.translation += velocity.0 * time.delta_secs();
    }
}

