use bevy::{
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_creeps::world::{State, WORLD_SIZE};
use bevy_creeps::ui::UIPlugin;
use bevy_creeps::gameplay::GameplayPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
                WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        canvas: Some(String::from("#game")),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
        ))
        .add_plugins(UIPlugin)
        .add_plugins(GameplayPlugin)
        .init_state::<State>()
        .add_systems(Startup, (setup_camera,))
        .add_systems(Update, check_restart.run_if(in_state(State::GameOver)))
        .run();
}


fn setup_camera(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scaling_mode = ScalingMode::AutoMin {
        min_width: WORLD_SIZE,
        min_height: WORLD_SIZE,
    };
    commands.spawn((Camera2d, projection));
}

fn check_restart(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut nextstate: ResMut<NextState<State>>,
) {
    if keyboard_input.pressed(KeyCode::KeyR) {
        nextstate.set(State::Playing)
    }
}
