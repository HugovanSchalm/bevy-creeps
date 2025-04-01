use std::time::Duration;

use bevy::{diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, math::bounding::{Aabb2d, IntersectsVolume}, prelude::*, render::camera::ScalingMode};
use rand::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum State {
    #[default]
    Playing,
    GameOver
}

const PLAYER_MAX_SPEED: f32 = 200.0;
const PLAYER_ACCELERATION: f32 = 50.0;
const PLAYER_DRAG: f32 = 50.0;
const PLAYER_SIZE: f32 = 20.0;
const WORLD_SIZE: f32 = 600.0;
const WORLD_BOUNDARY_VECTOR: Vec3 = Vec3::new(0.5 * WORLD_SIZE - 0.5 * PLAYER_SIZE, 0.5 * WORLD_SIZE - 0.5 * PLAYER_SIZE, 0.0);
const ENEMY_SPAWN_RADIUS: f32 = 400.0;
const ENEMY_SPAWN_TIME: f32 = 1.0;
const ENEMY_SIZE: f32 = 20.0;
const ENEMY_SPEED: f32 = 300.0;
const ENEMY_DESPAWN_RADIUS: f32 = 1000.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        //.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(EnemySpawnTimer(Timer::new(Duration::from_secs_f32(ENEMY_SPAWN_TIME), TimerMode::Repeating)))
        .insert_resource(ScoreTimer(Timer::new(Duration::from_secs(1), TimerMode::Repeating)))
        .insert_resource(Score(0))
        .init_state::<State>()
        .add_systems(Startup, (
                    setup_camera,
        ))
        .add_systems(FixedUpdate, (
                player_movement,
                spawn_enemies,
                apply_velocity,
                clamp_player,
                despawn_enemies,
                update_score,
                update_score_ui,
                collide,
            ).chain().run_if(in_state(State::Playing))
        )
        .add_systems(Update, check_restart.run_if(in_state(State::GameOver)))
        .add_systems(OnEnter(State::GameOver), 
            (
                    show_game_over,
                    remove_score_ui
                )
            )
        .add_systems(OnEnter(State::Playing), reset)
        .run();
}

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

#[derive(Resource)]
struct ScoreTimer(Timer);

#[derive(Resource)]
struct Score(u32);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct ScoreUI;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameOverUI;

#[derive(Component, Default)]
struct Velocity(Vec3);

fn setup_camera(
    mut commands: Commands,
) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scaling_mode = ScalingMode::AutoMin { min_width: WORLD_SIZE, min_height: WORLD_SIZE };
    commands.spawn(
        (
            Camera2d,
            projection,
        )
    );
}

fn reset(
    mut commands: Commands,
    player_entity: Option<Single<Entity, With<Player>>>,
    enemy_entities: Query<Entity, With<Enemy>>,
    game_over_ui_entity: Option<Single<Entity, With<GameOverUI>>>,
    mut score: ResMut<Score>,
) {
    score.0 = 0;

    if let Some(entity) = player_entity {
        commands.entity(*entity).despawn();
    }
    enemy_entities.iter().for_each(|entity| commands.entity(entity).despawn());
    if let Some(entity) = game_over_ui_entity {
        commands.entity(*entity).despawn_recursive();
    }

    commands.spawn(
        (
            Sprite::from_color(Color::srgb(0.0, 0.0, 1.0), Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
            Velocity::default(),
            Player,
        ),
    );

    commands.spawn(
        (
            ScoreUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),                
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..Default::default()
            }
        )
    ).with_children(|parent| {
        parent.spawn(
            Text::new("Score: ")
        );
        parent.spawn(
            (
                ScoreText,
                Text::new("0")
            )
        );
    });
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

fn apply_velocity(
    mut query: Query<(&Velocity, &mut Transform)>,
    time: Res<Time>,
) {
    for result in query.iter_mut() {
        let (velocity, mut transform) = result;
        transform.translation += velocity.0 * time.delta_secs();
    }
}

fn clamp_player(
    mut player_transform: Single<&mut Transform, With<Player>>
) {
    player_transform.translation = player_transform.translation.clamp(-WORLD_BOUNDARY_VECTOR, WORLD_BOUNDARY_VECTOR);
}

fn spawn_enemies(
    mut commands: Commands,
    mut timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
) {
    if timer.0.tick(time.delta()).finished() {
        let up = Vec3::new(0.0, 1.0, 0.0);

        let spawn_angle: f32 = rand::rng().random_range(0.0..std::f32::consts::TAU);
        let position = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), spawn_angle).mul_vec3(up) * ENEMY_SPAWN_RADIUS;

        let velocity_angle = spawn_angle + rand::rng().random_range(-0.2 * std::f32::consts::PI..0.2 * std::f32::consts::PI);
        let velocity = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), velocity_angle).mul_vec3(-up) * ENEMY_SPEED;
        commands.spawn(
    (
                Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::new(ENEMY_SIZE, ENEMY_SIZE)),
                Transform::from_translation(position),
                Velocity(velocity),
                Enemy,
            )
        );
    }
}

fn despawn_enemies(
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

fn show_game_over(
    mut commands: Commands,
    score: Res<Score>,
) {
    commands.spawn(
        (
            GameOverUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                display: Display::Flex,
                ..Default::default()
            }
        )
    ).with_children(|parent| {
        parent.spawn(
            Node::default()
        ).with_child(Text::new("Game Over"));
                
        parent.spawn(
            Node::default()
        ).with_child(Text::new(format!("Score: {}", score.0)));
    });
}

fn update_score(
    mut score_timer: ResMut<ScoreTimer>,
    mut score: ResMut<Score>,
    time: Res<Time>,
) {
    if score_timer.0.tick(time.delta()).finished() {
        score.0 += 1;
    }
}

fn update_score_ui(
    mut score_text: Single<&mut Text, With<ScoreText>>,
    score: Res<Score>
) {
    score_text.0 = format!("{}", score.0);
}

fn remove_score_ui(
    score_entity: Single<Entity, With<ScoreUI>>,
    mut commands: Commands,
) {
    commands.entity(*score_entity).despawn_recursive();
}

fn check_restart(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut nextstate: ResMut<NextState<State>>,
) {
    if keyboard_input.pressed(KeyCode::KeyR) {
        nextstate.set(State::Playing)
    }
}
