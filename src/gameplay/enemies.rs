use crate::gameplay::movement::Velocity;
use crate::world::State;
use bevy::prelude::*;
use bevy::utils::HashMap;
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
pub const ENEMY_DESPAWN_RADIUS: f32 = 401.0;

const UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        let mut spawntable = SpawnTable(HashMap::new());
        spawntable.0.insert(Enemy::Standard, 1);
        spawntable.0.insert(Enemy::Bullet, 1);
        spawntable.0.insert(Enemy::Cannon, 1);
        app.insert_resource(EnemySpawnTimer(Timer::new(
            Duration::from_secs_f32(2.0),
            TimerMode::Repeating,
        )))
        .insert_resource(spawntable)
        .add_systems(
            FixedUpdate,
            (spawn_enemies, despawn_out_of_bounds_enemies, handle_spawners).run_if(in_state(State::Playing)),
        )
        .add_systems(OnEnter(State::Playing), despawn_all_enemies);
    }
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Enemy {
    Standard,
    Bullet,
    Cannon,
}

impl Enemy {
    pub fn size(&self) -> f32 {
        match self {
            Enemy::Bullet => 10.0,
            Enemy::Cannon => 40.0,
            _ => 20.0
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Enemy::Bullet => Color::srgb(1.0, 0.5, 0.0),
            Enemy::Cannon => Color::srgb(0.5, 0.0, 1.0),
            _ => Color::srgb(1.0, 0.0, 0.0)
        }
    }
    pub fn speed(&self) -> f32 {
        match self {
            Enemy::Bullet => 450.0,
            Enemy::Cannon => 100.0,
            _ => 300.0
        }
    }
}

#[derive(Component)]
struct Spawner {
    enemy: Enemy,
    amount: usize,
    timer: Timer,
}

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

#[derive(Resource)]
struct SpawnTable(HashMap<Enemy, u32>);

impl SpawnTable {
    fn draw(&self) -> Enemy {
        let totalweight = self.0.values().sum();
        let random_weight = rand::thread_rng().gen_range(0..totalweight);
        let mut weightsum = 0;
        let mut iter = self.0.iter();
        while let Some((enemy, weight)) = iter.next() {
            weightsum += *weight;
            if weightsum > random_weight {
                return *enemy;
            }       
        };
        return Enemy::Standard;
    }
}

fn spawn_single_enemy(
    enemy: Enemy,
    position: Vec3,
    movement_angle: f32,
    commands: &mut Commands,
) {
    let direction = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), movement_angle);

    let size = enemy.size();

    let speed = enemy.speed();

    let color = enemy.color();

    let velocity = direction.mul_vec3(-UP) * speed;

    match enemy {
        Enemy::Cannon => commands.spawn((
            Sprite::from_color(
                color,
                Vec2::new(size, size),
            ),
            Transform::from_translation(position),
            Velocity::new(velocity),
            enemy,
            Spawner {
                enemy: Enemy::Bullet,
                amount: 12,
                timer: Timer::from_seconds(2.0, TimerMode::Repeating)
            },
        )),
        _ => commands.spawn((
            Sprite::from_color(
                color,
                Vec2::new(size, size),
            ),
            Transform::from_translation(position),
            Velocity::new(velocity),
            enemy,
        ))
    };
}

fn spawn_enemies(
    mut commands: Commands,
    mut timer: ResMut<EnemySpawnTimer>,
    spawntable: Res<SpawnTable>,
    time: Res<Time>,
    score: Res<Score>,
) {
    if timer.0.tick(time.delta()).finished() {
        let mut random = rand::thread_rng();
        let score = score.0 as f32;

        let spawn_angle: f32 = random.gen_range(0.0..TAU);
        let position = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), spawn_angle).mul_vec3(UP)
            * ENEMY_SPAWN_RADIUS;

        let movement_angle = spawn_angle + random.gen_range(-0.3..0.3);

        spawn_single_enemy(spawntable.draw(), position, movement_angle, &mut commands);

        let timer_distribution = Normal::new(
            (ENEMY_BASE_SPAWN_TIME_MEAN + score * ENEMY_DECREASE_SPAWN_TIME_MEAN) as f64,
            (ENEMY_BASE_SPAWN_TIME_STD + score * ENEMY_DECREASE_SPAWN_TIME_STD) as f64,
        )
        .unwrap();
        timer.0.set_duration(Duration::from_secs_f64(
            timer_distribution
                .sample(&mut rand::thread_rng())
                .max(ENEMY_MIN_SPAWN_TIME as f64),
        ));
    }
}

fn handle_spawners(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut Spawner)>,
    time: Res<Time>,
) {
    query.iter_mut().for_each(|(transform, mut spawner)| {
        if spawner.timer.tick(time.delta()).finished() {
            for i in 0..spawner.amount {
                let movement_angle = i as f32 * (TAU / spawner.amount as f32);
                spawn_single_enemy(spawner.enemy, transform.translation, movement_angle, &mut commands);
            }
        }
    })
}

fn despawn_out_of_bounds_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Enemy>>,
) {
    query.iter().for_each(|(entity, transform)| {
        if transform.translation.length() > ENEMY_DESPAWN_RADIUS {
            commands.entity(entity).despawn();
        }
    })
}

fn despawn_all_enemies(mut commands: Commands, enemy_entities: Query<Entity, With<Enemy>>) {
    enemy_entities
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
}
