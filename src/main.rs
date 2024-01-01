use bevy::prelude::*;

use crate::player::PlayerPlugin;
use crate::weapon::WeaponsPlugin;

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
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_plugins((PlayerPlugin, WeaponsPlugin))
        .run();
}
