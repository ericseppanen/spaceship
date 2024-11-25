use bevy::math::vec2;
use bevy::prelude::*;

use crate::weapon::{Weapon, WeaponFireEvent};
use crate::GameState;

const PLAYER_SPEED: f32 = 200.0;
const PLAYER_PROJECTILE_VELOCITY: f32 = 400.0;
const PLAYER_SPAWN_POSITION: Vec2 = vec2(0.0, -300.0);

#[derive(Resource)]
struct PlayerAssets {
    player_ship_image: Handle<Image>,
}

impl PlayerAssets {
    fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
        let player_ship_image = asset_server.load("red_ship.png");

        commands.insert_resource(PlayerAssets { player_ship_image });
    }

    /// Create a Sprite for the player ship.
    fn player_ship(&self) -> Sprite {
        Sprite::from_image(self.player_ship_image.clone_weak())
    }
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: PLAYER_SPEED,
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    sprite: Sprite,
    transform: Transform,
    weapon: Weapon,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        let aim = Vec2 {
            x: 0.0,
            y: PLAYER_PROJECTILE_VELOCITY,
        };
        let weapon = Weapon::new(aim, 0.25);
        Self {
            player: Default::default(),
            sprite: Default::default(),
            transform: Default::default(),
            weapon,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerSpawnEvent>()
            .add_systems(Startup, PlayerAssets::load)
            .add_systems(
                Update,
                (spawn_player, player_movement).run_if(in_state(GameState::Playing)),
            );
    }
}

/// Calibrate the analog inputs a bit.
///
/// Add a dead zone near 0.0, and set the max output at 90% deflection.
fn tune_analog(input: f32) -> f32 {
    let sign = input.signum();
    let mut value = input.abs();
    // scale (0.05, 0.90) to (0.0, 1.0)
    value -= 0.05;
    value /= 0.85;
    value.clamp(0.0, 1.0) * sign
}

pub fn player_movement(
    mut players: Query<(&mut Transform, &Player, Entity)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    time: Res<Time<Virtual>>,
    mut event_sender: EventWriter<WeaponFireEvent>,
) {
    // for ScanCode(code) in keyboard.get_pressed() {
    //     info!("scancode {code:#x}");
    // }

    // We could iterate, but since we're hard-coding the
    // keyboard controls, that wouldn't make sense.
    let Ok((mut transform, player, entity)) = players.get_single_mut() else {
        return;
    };

    let move_delta = player.speed * time.delta_secs();

    let mut analog_x = None;
    let mut analog_y = None;
    let mut fire_button = false;
    // FIXME: gracefully handle >1 gamepads
    if let Some(gamepad) = gamepads.iter().next() {
        let analog = gamepad.left_stick().map(tune_analog);
        analog_x = Some(analog.x);
        analog_y = Some(analog.y);
        fire_button = gamepad.digital().just_pressed(GamepadButton::South);
    }

    if keyboard.pressed(KeyCode::ArrowUp) {
        transform.translation.y += move_delta;
    } else if keyboard.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= move_delta;
    } else if let Some(y_value) = analog_y {
        transform.translation.y += move_delta * y_value;
    }

    if keyboard.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= move_delta;
    } else if keyboard.pressed(KeyCode::ArrowRight) {
        transform.translation.x += move_delta;
    } else if let Some(x_value) = analog_x {
        transform.translation.x += move_delta * x_value;
    }

    const PLAYER_BOUNDS: Vec2 = Vec2 { x: 180.0, y: 380.0 };

    // Don't let the ship travel offscreen.
    let extents = PLAYER_BOUNDS.extend(0.0);
    transform.translation = transform.translation.min(extents).max(-extents);

    if keyboard.just_pressed(KeyCode::Space) || fire_button {
        event_sender.send(WeaponFireEvent(entity));
    }
}

/// Spawn the player ship.
#[derive(Event)]
pub struct PlayerSpawnEvent;

fn spawn_player(
    mut commands: Commands,
    mut event: EventReader<PlayerSpawnEvent>,
    assets: Res<PlayerAssets>,
    players: Query<Entity, With<Player>>,
) {
    // Pop all events from the queue.
    let Some(_) = event.read().last() else {
        return;
    };

    // Check if the player has already spawned.
    // This will be true if we just bumped the level.
    if players.get_single().is_ok() {
        return;
    }

    info!("spawn player");

    let sprite = assets.player_ship();
    let transform = Transform::from_translation(PLAYER_SPAWN_POSITION.extend(0.0));
    let player = PlayerBundle {
        sprite,
        transform,
        ..default()
    };

    commands.spawn(player);
}
