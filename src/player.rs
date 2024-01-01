use bevy::prelude::*;

use crate::scancodes;
use crate::weapon::{Weapon, WeaponFireEvent};

const PLAYER_SPEED: f32 = 200.0;
const PLAYER_PROJECTILE_VELOCITY: f32 = 400.0;

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
    sprite: SpriteBundle,
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
            weapon,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, player_movement);
    }
}

fn player_movement(
    mut players: Query<(&mut Transform, &Player, Entity)>,
    input: Res<Input<ScanCode>>,
    time: Res<Time>,
    mut event_sender: EventWriter<WeaponFireEvent>,
) {
    // for ScanCode(code) in input.get_pressed() {
    //     info!("scancode {code:#x}");
    // }

    // We could iterate, but since we're hard-coding the
    // keyboard controls, that wouldn't make sense.
    let Ok((mut transform, player, entity)) = players.get_single_mut() else {
        return;
    };

    let move_delta = player.speed * time.delta_seconds();

    if input.pressed(scancodes::UP) {
        transform.translation.y += move_delta;
    }
    if input.pressed(scancodes::DOWN) {
        transform.translation.y -= move_delta;
    }
    if input.pressed(scancodes::LEFT) {
        transform.translation.x += move_delta;
    }
    if input.pressed(scancodes::RIGHT) {
        transform.translation.x -= move_delta;
    }

    const PLAYER_BOUNDS: Vec2 = Vec2 { x: 180.0, y: 380.0 };

    // Don't let the ship travel offscreen.
    let extents = PLAYER_BOUNDS.extend(0.0);
    transform.translation = transform.translation.min(extents).max(-extents);

    if input.just_pressed(scancodes::SPACE) {
        event_sender.send(WeaponFireEvent(entity));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("red_ship.png");
    let transform = Transform::from_translation(Vec3::new(0.0, -300.0, 0.0));

    let sprite = SpriteBundle {
        texture,
        transform,
        ..default()
    };
    let player = PlayerBundle {
        sprite,
        ..default()
    };

    commands.spawn(player);
}
