use crate::world::State;
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (apply_acceleration, apply_velocity)
                .chain()
                .run_if(in_state(State::Playing)),
        );
    }
}

#[derive(Component, Default)]
pub struct Velocity {
    pub value: Vec3,
    max: f32,
}

impl Velocity {
    pub fn new(value: Vec3, max: f32) -> Self {
        Velocity { value, max }
    }
}

#[derive(Component, Default)]
pub struct Acceleration {
    pub direction: Vec3,
    pub amount: f32,
}

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for result in query.iter_mut() {
        let (velocity, mut transform) = result;
        transform.translation += velocity.value * time.delta_secs();
    }
}

fn apply_acceleration(mut query: Query<(&Acceleration, &mut Velocity)>, time: Res<Time>) {
    for (acceleration, mut velocity) in query.iter_mut() {
        velocity.value = velocity.value.lerp(
            acceleration.direction.normalize_or_zero() * velocity.max,
            1.0 - f32::exp(-acceleration.amount * time.delta_secs()),
        );
    }
}
