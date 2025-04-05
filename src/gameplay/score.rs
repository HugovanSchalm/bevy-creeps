use crate::world::State;
use bevy::prelude::*;
use std::time::Duration;

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScoreTimer(Timer::new(
            Duration::from_secs(1),
            TimerMode::Repeating,
        )))
        .insert_resource(Score(0))
        .add_systems(FixedUpdate, update_score.run_if(in_state(State::Playing)))
        .add_systems(OnEnter(State::Playing), reset_score);
    }
}

#[derive(Resource)]
struct ScoreTimer(Timer);

#[derive(Resource)]
pub struct Score(pub u32);

fn reset_score(mut score: ResMut<Score>, mut timer: ResMut<ScoreTimer>) {
    score.0 = 0;
    timer.0.reset();
}

fn update_score(mut score_timer: ResMut<ScoreTimer>, mut score: ResMut<Score>, time: Res<Time>) {
    if score_timer.0.tick(time.delta()).finished() {
        score.0 += 1;
    }
}
