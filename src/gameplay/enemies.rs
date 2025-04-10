use crate::gameplay::movement::{Acceleration, Velocity};
use crate::gameplay::player::Player;
use crate::world::{State, WORLD_SIZE};
use bevy::prelude::*;
use bevy::utils::HashMap;
use rand::Rng;
use std::f32::consts::TAU;
use std::time::Duration;

use super::score::ScoreIncreasedEvent;

pub const ENEMY_SPAWN_RADIUS: f32 = WORLD_SIZE + 10.0;
pub const ENEMY_DESPAWN_RADIUS: f32 = ENEMY_SPAWN_RADIUS + 1.0;

const UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        let initial_time_between_spawns = Duration::from_secs_f32(2.0);
        app.insert_resource(EnemySpawnTimer(Timer::new(
            initial_time_between_spawns,
            TimerMode::Repeating,
        )))
        .insert_resource(SpawnParameters::default())
        .add_systems(
            FixedUpdate,
            (
                spawn_enemies,
                despawn_out_of_bounds_enemies,
                handle_shooting,
                handle_heatseeker_acceleration,
                handle_heatseeker_destruction,
                increase_difficulty,
            )
                .run_if(in_state(State::Playing)),
        )
        .add_systems(
            OnEnter(State::Playing),
            (despawn_all_enemies, reset_difficulty),
        );
    }
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Enemy {
    Standard,
    Bullet,
    Cannon,
    Rocket,
    RocketShip,
}

impl Enemy {
    pub fn size(&self) -> f32 {
        match self {
            Enemy::Bullet | Enemy::Rocket => 10.0,
            Enemy::Cannon | Enemy::RocketShip => 40.0,
            _ => 20.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Enemy::Bullet => Color::srgb(5.0, 2.5, 0.0),
            Enemy::Cannon => Color::srgb(2.5, 0.0, 5.0),
            Enemy::Rocket => Color::srgb(0.0, 5.0, 0.0),
            Enemy::RocketShip => Color::srgb(0.0, 2.5, 5.0),
            _ => Color::srgb(5.0, 0.0, 0.0),
        }
    }
    pub fn speed(&self) -> f32 {
        match self {
            Enemy::Bullet | Enemy::Rocket => 450.0,
            Enemy::Cannon | Enemy::RocketShip => 200.0,
            _ => 300.0,
        }
    }
}

#[derive(Component)]
struct ShootTimer(Timer);

#[derive(Component)]
struct HeatSeeker {
    alive_timer: Timer,
}

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

#[derive(Resource)]
/// Parameters for generic spawning (so not shooting).
/// This is what determines the difficulty along with [SpawnTable].
struct SpawnParameters {
    time_between_spawns: Duration,
    min_time_between_spawns: Duration,
    probability_spawn_another: f64,
    max_probability_spawn_another: f64,
    max_spawns: u32,
    spawn_table: HashMap<Enemy, u32>,
}

impl SpawnParameters {
    fn draw_enemy(&self) -> Enemy {
        let totalweight = self.spawn_table.values().sum();
        let randomweight = rand::thread_rng().gen_range(1..=totalweight);
        let mut weightsum = 0;
        let mut iter = self.spawn_table.iter();
        while let Some((enemy, weight)) = iter.next() {
            weightsum += *weight;
            if weightsum >= randomweight {
                return *enemy;
            }
        }
        eprintln!("Could not draw enemy! Just spawning a standard enemy.");
        return Enemy::Standard;
    }
}

impl Default for SpawnParameters {
    fn default() -> Self {
        let mut spawn_parameters = SpawnParameters {
            time_between_spawns: Duration::from_secs(2),
            min_time_between_spawns: Duration::from_millis(300),
            max_spawns: 1,
            probability_spawn_another: 0.0,
            max_probability_spawn_another: 0.8,
            spawn_table: HashMap::new(),
        };
        spawn_parameters.spawn_table.insert(Enemy::Standard, 25);
        spawn_parameters
    }
}

fn spawn_single_enemy(enemy: Enemy, position: Vec3, direction: Vec3, commands: &mut Commands) {
    let size = enemy.size();
    let speed = enemy.speed();
    let color = enemy.color();

    let velocity = direction * speed;

    let entity = commands
        .spawn((
            Sprite::from_color(color, Vec2::new(size, size)),
            Transform::from_translation(position),
            Velocity::new(velocity, velocity.length()),
            enemy,
        ))
        .id();

    match enemy {
        Enemy::Cannon => {
            commands
                .entity(entity)
                .insert(ShootTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        }
        Enemy::RocketShip => {
            commands
                .entity(entity)
                .insert(ShootTimer(Timer::from_seconds(3.0, TimerMode::Repeating)));
        }
        Enemy::Rocket => {
            commands.entity(entity).insert((
                HeatSeeker {
                    alive_timer: Timer::new(Duration::from_secs(5), TimerMode::Once),
                },
                Acceleration {
                    direction: Vec3::ZERO,
                    amount: 1.0,
                },
            ));
        }
        _ => {}
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut timer: ResMut<EnemySpawnTimer>,
    spawn_parameters: Res<SpawnParameters>,
    time: Res<Time>,
) {
    if timer.0.tick(time.delta()).finished() {
        let mut random = rand::thread_rng();

        let mut spawned = 0;
        while spawned < spawn_parameters.max_spawns {
            let spawn_angle: f32 = random.gen_range(0.0..TAU);
            let position = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), spawn_angle)
                .mul_vec3(UP)
                * ENEMY_SPAWN_RADIUS;

            let movement_angle = spawn_angle + random.gen_range(-0.3..0.3);
            let direction = Quat::from_axis_angle(Vec3::Z, movement_angle).mul_vec3(Vec3::NEG_Y);

            spawn_single_enemy(
                spawn_parameters.draw_enemy(),
                position,
                direction,
                &mut commands,
            );

            if random.gen_bool(1.0 - spawn_parameters.probability_spawn_another) {
                break;
            }

            spawned += 1;
        }

        timer.0.set_duration(spawn_parameters.time_between_spawns);
    }
}

fn handle_shooting(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut ShootTimer, &Enemy)>,
    player_transform_query: Option<Single<&GlobalTransform, With<Player>>>, // This limits parallelization and is only needed for rocketship so maybe change
    time: Res<Time>,
) {
    for (transform, mut timer, enemy) in query.iter_mut() {
        if timer.0.tick(time.delta()).finished() {
            match enemy {
                Enemy::Cannon => {
                    for i in 0..12 {
                        let movement_angle = i as f32 * (TAU / 12.0 as f32);
                        let direction =
                            Quat::from_axis_angle(Vec3::Z, movement_angle).mul_vec3(Vec3::NEG_Y);
                        spawn_single_enemy(
                            Enemy::Bullet,
                            transform.translation,
                            direction,
                            &mut commands,
                        );
                    }
                }
                Enemy::RocketShip => {
                    let direction = match player_transform_query {
                        Some(ref player_transform) => transform
                            .looking_at(player_transform.translation(), Vec3::Y)
                            .forward()
                            .normalize_or(Vec3::Y),
                        None => Vec3::Y,
                    };
                    spawn_single_enemy(
                        Enemy::Rocket,
                        transform.translation,
                        direction,
                        &mut commands,
                    );
                }
                _ => {}
            }
        }
    }
}

fn handle_heatseeker_acceleration(
    mut heatseeker_accelerations: Query<(&Transform, &mut Acceleration), With<HeatSeeker>>,
    player_transform: Option<Single<&Transform, With<Player>>>,
) {
    if player_transform.is_none() {
        return;
    }

    let player_transform = player_transform.unwrap();

    for (transform, mut acceleration) in heatseeker_accelerations.iter_mut() {
        acceleration.direction = transform
            .looking_at(player_transform.translation, Vec3::Y)
            .forward()
            .into();
    }
}

fn handle_heatseeker_destruction(
    mut heatseekers: Query<(Entity, &mut HeatSeeker)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut heatseeker) in heatseekers.iter_mut() {
        if heatseeker.alive_timer.tick(time.delta()).finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn increase_difficulty(
    mut spawn_parameters: ResMut<SpawnParameters>,
    mut score_increased_event: EventReader<ScoreIncreasedEvent>,
) {
    for event in score_increased_event.read() {
        let new_score = event.get_new_score();
        spawn_parameters.probability_spawn_another = spawn_parameters
            .max_probability_spawn_another
            .min(spawn_parameters.probability_spawn_another + 0.03);
        spawn_parameters.time_between_spawns = spawn_parameters
            .min_time_between_spawns
            .max(spawn_parameters.time_between_spawns - Duration::from_millis(10));

        spawn_parameters.max_spawns = 1 + new_score / 30;

        spawn_parameters
            .spawn_table
            .insert(Enemy::Cannon, new_score / 10);

        spawn_parameters
            .spawn_table
            .insert(Enemy::RocketShip, new_score / 15);
    }
}

fn reset_difficulty(mut spawn_parameters: ResMut<SpawnParameters>) {
    *spawn_parameters = SpawnParameters::default();
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
