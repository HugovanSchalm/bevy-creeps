use crate::gameplay::movement::Velocity;
use crate::world::State;
use bevy::prelude::*;
use rand::Rng;
use rand::distributions::Distribution;
use statrs::distribution::Normal;
use std::f32::consts::TAU;
use std::time::Duration;

use super::score::Score;

pub const ENEMY_SPAWN_RADIUS: f32 = 400.0;
pub const ENEMY_BASE_SPAWN_TIME_MEAN: f32 = 1.0;
pub const ENEMY_BASE_SPAWN_TIME_STD: f32 = 1.0;
pub const ENEMY_DECREASE_SPAWN_TIME_MEAN: f32 = 0.01;
pub const ENEMY_DECREASE_SPAWN_TIME_STD: f32 = 0.01;
pub const ENEMY_MIN_SPAWN_TIME: f32 = 0.01;
pub const ENEMY_SIZE: f32 = 20.0;
pub const ENEMY_BASE_SPEED_MEAN: f32 = 300.0;
pub const ENEMY_BASE_SPEED_STD: f32 = 300.0;
pub const ENEMY_INCREASE_SPEED_MEAN: f32 = 0.01;
pub const ENEMY_INCREASE_SPEED_STD: f32 = 0.01;
pub const ENEMY_DESPAWN_RADIUS: f32 = 1000.0;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnTimer(Timer::new(
            Duration::from_secs_f32(2.0),
            TimerMode::Repeating,
        )))
        .add_systems(
            FixedUpdate,
            (spawn_enemies, despawn_out_of_bounds_enemies).run_if(in_state(State::Playing)),
        )
        .add_systems(OnEnter(State::Playing), despawn_all_enemies);
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

fn spawn_enemies(
    mut commands: Commands,
    mut timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
    score: Res<Score>,
) {
    if timer.0.tick(time.delta()).finished() {
        let mut random = rand::thread_rng();
        let score = score.0 as f32;
        let up = Vec3::new(0.0, 1.0, 0.0);

        let spawn_angle: f32 = random.gen_range(0.0..TAU);
        let position = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), spawn_angle).mul_vec3(up)
            * ENEMY_SPAWN_RADIUS;

        let velocity_angle = spawn_angle + random.gen_range(-0.3..0.3);
        let velocity_distribution = Normal::new(
            (ENEMY_BASE_SPEED_MEAN + score * ENEMY_INCREASE_SPEED_MEAN) as f64,
            (ENEMY_BASE_SPEED_STD + score * ENEMY_INCREASE_SPEED_STD) as f64,
        )
        .unwrap();
        let velocity = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), velocity_angle)
            .mul_vec3(-up)
            * velocity_distribution.sample(&mut random).abs() as f32;
        commands.spawn((
            Sprite::from_color(
                Color::srgb(1.0, 0.0, 0.0),
                Vec2::new(ENEMY_SIZE, ENEMY_SIZE),
            ),
            Transform::from_translation(position),
            Velocity::new(velocity),
            Enemy,
        ));

        let timer_distribution = Normal::new(
            (ENEMY_BASE_SPAWN_TIME_MEAN + score * ENEMY_DECREASE_SPAWN_TIME_MEAN) as f64,
            (ENEMY_BASE_SPAWN_TIME_STD + score * ENEMY_DECREASE_SPAWN_TIME_STD) as f64,
        )
        .unwrap();
        timer.0.set_duration(Duration::from_secs_f64(
            timer_distribution
                .sample(&mut random)
                .max(ENEMY_MIN_SPAWN_TIME as f64),
        ));
    }
}

fn despawn_out_of_bounds_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Enemy>>,
) {
    for result in query.iter() {
        let (entity, transform) = result;
        if transform.translation.length() > ENEMY_DESPAWN_RADIUS {
            commands.entity(entity).despawn();
        }
    }
}

fn despawn_all_enemies(mut commands: Commands, enemy_entities: Query<Entity, With<Enemy>>) {
    enemy_entities
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
}
