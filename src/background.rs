use bevy::math::vec3;
use bevy::prelude::*;

const MOVEMENT_RATE: f32 = 40.0;
// FIXME: determine this at runtime.
const BG_REPEAT_HEIGHT: f32 = 800.0;

pub struct BgPlugin;

impl Plugin for BgPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_background)
            .add_systems(Update, move_background);
    }
}

#[derive(Component)]
struct Background;

/// Spawn a sprite with the background image
fn init_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image = asset_server.load("stars1.png");
    let transform = Transform::from_translation(vec3(0.0, BG_REPEAT_HEIGHT / 2.0, -100.0));

    commands.spawn((Sprite::from_image(image), transform, Background));
}

/// Slowly move the background image
fn move_background(time: Res<Time>, mut query: Query<&mut Transform, With<Background>>) {
    let mut transform = query.single_mut();
    let mut new_y = transform.translation.y - MOVEMENT_RATE * time.delta_secs();

    // The background image is tiled 2x so the top half and bottom half are identical.
    // The background will scroll from +BG_REPEAT_HEIGHT/2 TO -BG_REPEAT_HEIGHT/2
    // at which point we will reset it by exactly BG_REPEAT_HEIGHT, so the change
    // is invisible.

    if new_y < -BG_REPEAT_HEIGHT / 2.0 {
        new_y += BG_REPEAT_HEIGHT;
    }
    transform.translation.y = new_y;
}
