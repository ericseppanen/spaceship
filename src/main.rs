use bevy::prelude::*;
use bevy::window::PresentMode;

use crate::enemy::EnemyPlugin;
use crate::player::PlayerPlugin;
use crate::weapon::WeaponsPlugin;

mod enemy;
mod player;
mod scancodes;
mod weapon;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                // default_nearest() prevents blurring of pixel art
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Spaceship!".into(),
                        resolution: (400.0, 800.0).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_systems(Startup, setup)
        .add_plugins((PlayerPlugin, WeaponsPlugin, EnemyPlugin))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
