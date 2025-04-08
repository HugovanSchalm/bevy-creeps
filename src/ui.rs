use crate::gameplay::score::Score;
use crate::world::State;
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (update_score_ui).run_if(in_state(State::Playing)),
        )
        .add_systems(
            OnEnter(State::Playing),
            (remove_game_over_ui, create_score_ui),
        )
        .add_systems(
            OnEnter(State::GameOver),
            (create_game_over_ui, remove_score_ui),
        );
    }
}
#[derive(Component)]
struct GameOverUI;

fn create_game_over_ui(mut commands: Commands, score: Res<Score>) {
    commands
        .spawn((
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
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(Node::default())
                .with_child(Text::new("Game Over"));

            parent
                .spawn(Node::default())
                .with_child(Text::new("Press <R> to restart"));

            parent
                .spawn(Node::default())
                .with_child(Text::new(format!("Score: {}", score.0)));
        });
}

fn remove_game_over_ui(
    mut commands: Commands,
    game_over_ui_entity: Option<Single<Entity, With<GameOverUI>>>,
) {
    if let Some(entity) = game_over_ui_entity {
        commands.entity(*entity).despawn_recursive();
    }
}

#[derive(Component)]
struct ScoreUI;

#[derive(Component)]
struct ScoreText;

fn create_score_ui(mut commands: Commands) {
    commands
        .spawn((
            ScoreUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("Score: "));
            parent.spawn((ScoreText, Text::new("0")));
        });
}
fn update_score_ui(mut score_text: Single<&mut Text, With<ScoreText>>, score: Res<Score>) {
    score_text.0 = format!("{}", score.0);
}

fn remove_score_ui(score_entity: Option<Single<Entity, With<ScoreUI>>>, mut commands: Commands) {
    if let Some(entity) = score_entity {
        commands.entity(*entity).despawn_recursive();
    }
}
