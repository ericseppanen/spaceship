use bevy::math::vec2;
use bevy::prelude::*;

use crate::scancodes;
use crate::weapon::{Weapon, WeaponFireEvent};

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

    /// Create a SpriteBundle for the player ship at the spawn position.
    fn player_ship(&self) -> SpriteBundle {
        let transform = Transform::from_translation(PLAYER_SPAWN_POSITION.extend(0.0));
        SpriteBundle {
            texture: self.player_ship_image.clone_weak(),
            transform,
            ..default()
        }
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
        app.add_event::<PlayerSpawnEvent>()
            .add_systems(Startup, PlayerAssets::load)
            .add_systems(Update, (spawn_player, player_movement));
    }
}

pub fn player_movement(
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
    let player = PlayerBundle {
        sprite,
        ..default()
    };

    commands.spawn(player);
}
