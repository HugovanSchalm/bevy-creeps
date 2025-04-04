use bevy::prelude::*;
use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use crate::world::{State, WORLD_BOUNDARY_VECTOR};
use crate::gameplay::enemies::{Enemy, ENEMY_SIZE};
use crate::gameplay::movement::Velocity;

pub const PLAYER_MAX_SPEED: f32 = 200.0;
pub const PLAYER_ACCELERATION: f32 = 50.0;
pub const PLAYER_DRAG: f32 = 50.0;
pub const PLAYER_SIZE: f32 = 20.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.
            add_systems(FixedUpdate, (
                    player_movement,
                    clamp_player,
                    collide,
            ).chain().run_if(in_state(State::Playing)))
            .add_systems(OnEnter(State::Playing), spawn_player);
    }
}

#[derive(Component)]
pub struct Player;

fn spawn_player(
    mut commands: Commands,
    player_entity: Option<Single<Entity, With<Player>>>,
) {
    if let Some(entity) = player_entity {
        commands.entity(*entity).despawn();
    }

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.0, 0.0, 1.0),
            Vec2::new(PLAYER_SIZE, PLAYER_SIZE),
        ),
        Velocity::default(),
        Player,
    ));
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_velocity: Single<&mut Velocity, With<Player>>,
    time: Res<Time>,
) {
    let mut input_vector = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        input_vector.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        input_vector.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        input_vector.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        input_vector.y -= 1.0;
    }

    input_vector = input_vector.normalize_or_zero();
    if input_vector.x.abs() > 0.0 || input_vector.y.abs() > 0.0 {
        player_velocity.0 = player_velocity.0.lerp(input_vector * PLAYER_MAX_SPEED, 1.0 - f32::exp(-PLAYER_ACCELERATION * time.delta_secs()));
    } else {
        player_velocity.0 = player_velocity.0.lerp(Vec3::ZERO, 1.0 - f32::exp(-PLAYER_DRAG * time.delta_secs()));
    }
}

fn clamp_player(
    mut player_transform: Single<&mut Transform, With<Player>>
) {
    player_transform.translation = player_transform.translation.clamp(-WORLD_BOUNDARY_VECTOR, WORLD_BOUNDARY_VECTOR);
}

fn collide (
    mut nextstate: ResMut<NextState<State>>,
    player_transform: Single<&Transform, With<Player>>,
    enemy_transforms: Query<&Transform, With<Enemy>>
) {
    for enemy_transform in enemy_transforms.iter(){
        let player_aabb = Aabb2d::new(player_transform.translation.xy(), Vec2::new(0.5 * PLAYER_SIZE, 0.5 * PLAYER_SIZE));
        let enemy_aabb = Aabb2d::new(enemy_transform.translation.xy(), Vec2::new(0.5 * ENEMY_SIZE, 0.5 * ENEMY_SIZE));
        if player_aabb.intersects(&enemy_aabb) {
            nextstate.set(State::GameOver);
        }
    }
        
}
