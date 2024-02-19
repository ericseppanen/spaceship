use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::camera::{ScalingMode, Viewport};
use bevy::window::{PresentMode, WindowResized, WindowResolution};

use crate::background::BgPlugin;
use crate::collide::CollisionPlugin;
use crate::enemy::EnemyPlugin;
use crate::level::LevelPlugin;
use crate::player::PlayerPlugin;
use crate::ui::UiPlugin;
use crate::weapon::WeaponsPlugin;

mod background;
mod collide;
mod enemy;
mod level;
mod player;
mod ui;
mod weapon;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, States)]
enum GameState {
    #[default]
    Idle,
    Playing,
    Paused,
}

fn main() {
    // Allow a canvas name to be specified on the commandline,
    // for compatibility with trunk.
    let canvas = option_env!("canvas_name").map(ToOwned::to_owned);
    dbg!(&canvas);

    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::hex("010101").unwrap()))
        .add_plugins(
            DefaultPlugins
                // default_nearest() prevents blurring of pixel art
                .set(ImagePlugin::default_linear())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas,
                        title: "Spaceship!".into(),
                        resolution: WindowResolution::new(400.0, 800.0),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, window_resize)
        .add_plugins((
            BgPlugin,
            PlayerPlugin,
            WeaponsPlugin,
            EnemyPlugin,
            CollisionPlugin,
            LevelPlugin,
            UiPlugin,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.camera.viewport = Some(Viewport {
        physical_size: (400, 800).into(),
        ..default()
    });

    camera.projection.scaling_mode = ScalingMode::FixedVertical(800.0);

    commands.spawn(camera);
}

// If the window gets resized, we need to update the camera viewport.
fn window_resize(mut resize_reader: EventReader<WindowResized>, mut query: Query<&mut Camera>) {
    if let Some(event) = resize_reader.read().last() {
        let mut camera = query.single_mut();
        let viewport = camera.viewport.as_mut().expect("couldn't find viewport");

        let mut physical_position = UVec2::ZERO;

        let physical_size = if event.height > 2.0 * event.width {
            // A very tall window is limited by the window width.
            let width = event.width;
            let height = event.width * 2.0;
            physical_position.y = ((event.height - height) / 2.0) as u32;
            (width as u32, height as u32)
        } else {
            // A very wide window is limited by the window height.
            let height = event.height;
            let width = event.height / 2.0;
            physical_position.x = ((event.width - width) / 2.0) as u32;
            (width as u32, height as u32)
        };

        viewport.physical_size = physical_size.into();
        viewport.physical_position = physical_position;
    }
}
