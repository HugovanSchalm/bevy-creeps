use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use crate::gameplay::movement::Velocity;
use crate::world::State;
use std::f32::consts::TAU;
use std::time::Duration;

pub const ENEMY_SPAWN_RADIUS: f32 = 400.0;
pub const ENEMY_SPAWN_TIME: f32 = 1.0;
pub const ENEMY_SIZE: f32 = 20.0;
pub const ENEMY_SPEED: f32 = 300.0;
pub const ENEMY_DESPAWN_RADIUS: f32 = 1000.0;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(EnemySpawnTimer(Timer::new(Duration::from_secs_f32(ENEMY_SPAWN_TIME), TimerMode::Repeating)))
            .insert_resource(RandomSource(ChaCha8Rng::seed_from_u64(12345)))
            .add_systems(FixedUpdate, (
                spawn_enemies,
                despawn_out_of_bounds_enemies,
            ).run_if(in_state(State::Playing)))
            .add_systems(OnEnter(State::Playing), despawn_all_enemies);
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

#[derive(Resource)]
struct RandomSource(ChaCha8Rng);

fn spawn_enemies(
    mut commands: Commands,
    mut timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
    mut random: ResMut<RandomSource>
) {
    if timer.0.tick(time.delta()).finished() {
        let up = Vec3::new(0.0, 1.0, 0.0);

        let spawn_angle: f32 = random.0.random_range(0.0..TAU);
        let position = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), spawn_angle).mul_vec3(up) * ENEMY_SPAWN_RADIUS;

        let velocity_angle = spawn_angle + random.0.random_range(-0.3..0.3);
        let velocity = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), velocity_angle).mul_vec3(-up) * ENEMY_SPEED;
        commands.spawn(
    (
                Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::new(ENEMY_SIZE, ENEMY_SIZE)),
                Transform::from_translation(position),
                Velocity::new(velocity),
                Enemy,
            )
        );
    }
}

fn despawn_out_of_bounds_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Enemy>>
) {
    for result in query.iter() {
        let (entity, transform) = result;
        if transform.translation.length() > ENEMY_DESPAWN_RADIUS {
            commands.entity(entity).despawn();
        }
    }
}

fn despawn_all_enemies(
    mut commands: Commands,
    enemy_entities: Query<Entity, With<Enemy>>
) {
    enemy_entities.iter().for_each(|entity| commands.entity(entity).despawn());
}
