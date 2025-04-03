use bevy::prelude::*;

mod movement;
mod enemies;
pub mod player;
pub mod score;

pub struct GameplayPlugin; 

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                    movement::MovementPlugin,
                    enemies::EnemyPlugin,
                    player::PlayerPlugin,
                    score::ScorePlugin,
            ));
    }
}
