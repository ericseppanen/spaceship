use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;

use crate::player::{player_movement, Player};

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WeaponFireEvent>()
            .add_systems(Startup, WeaponAssets::load)
            .add_systems(Update, (charge_weapons, move_projectiles))
            .add_systems(Update, fire_weapon.after(player_movement));
    }
}

#[derive(Resource)]
struct WeaponAssets {
    player_weapon_sound: Handle<AudioSource>,
    player_projectile_image: Handle<Image>,
    enemy_projectile_image: Handle<Image>,
}

impl WeaponAssets {
    fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
        let player_weapon_sound = asset_server.load("shoot1.wav");
        let player_projectile_image = asset_server.load("green_torpedo.png");
        let enemy_projectile_image = asset_server.load("blue_torpedo.png");

        commands.insert_resource(WeaponAssets {
            player_weapon_sound,
            player_projectile_image,
            enemy_projectile_image,
        });
    }

    /// Create an AudioBundle that will play the weapon sound.
    fn weapon_audio(&self, commands: &mut Commands) {
        commands.spawn((
            AudioPlayer(self.player_weapon_sound.clone_weak()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::new(0.2),
                ..default()
            },
        ));
    }
}

#[derive(Component)]
pub struct Projectile {
    pub velocity_vector: Vec2,
    /// Player projectiles can't hurt players; enemy projectiles can't hurt enemies.
    pub player: bool,
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    projectile: Projectile,
    sprite: Sprite,
    transform: Transform,
}

#[derive(Component)]
pub struct Weapon {
    // Vector determines the direction and velocity of the projectile.
    aim_vector: Vec2,
    // Time in seconds to recharge after a shot.
    ready_timer: Timer,
}

impl Weapon {
    pub fn new(aim_vector: Vec2, recharge_time: f32) -> Self {
        let ready_timer = Timer::from_seconds(recharge_time, TimerMode::Once);
        Self {
            aim_vector,
            ready_timer,
        }
    }
}

#[derive(Event)]
pub struct WeaponFireEvent(pub Entity);

/// Handle the `WeaponFireEvent`
fn fire_weapon(
    mut commands: Commands,
    mut event: EventReader<WeaponFireEvent>,
    mut query: Query<(&mut Weapon, &Transform, Option<&Player>)>,
    assets: Res<WeaponAssets>,
) {
    // Ignore multiple fire events.
    for event in event.read() {
        let (mut weapon, transform, player) =
            query.get_mut(event.0).expect("missing weapon in entity");

        let timer = &mut weapon.ready_timer;
        if timer.finished() {
            timer.reset();
        } else {
            // weapon is still charging, do nothing.
            return;
        }

        let sprite = if player.is_some() {
            let image = assets.player_projectile_image.clone_weak();
            Sprite { image, ..default() }
        } else {
            let image = assets.enemy_projectile_image.clone_weak();
            Sprite {
                flip_y: true,
                image,
                ..default()
            }
        };

        let projectile = Projectile {
            velocity_vector: weapon.aim_vector,
            player: player.is_some(),
        };
        let bundle = ProjectileBundle {
            projectile,
            sprite,
            transform: *transform,
        };

        commands.spawn(bundle);
        assets.weapon_audio(&mut commands);
    }
}

/// Advance time in weapons timers.
fn charge_weapons(mut query: Query<&mut Weapon>, time: Res<Time>) {
    for mut weapon in &mut query {
        weapon.ready_timer.tick(time.delta());
    }
}

/// Move projectiles in a straight line.
fn move_projectiles(
    mut commands: Commands,
    mut query: Query<(&Projectile, &mut Transform, Entity)>,
    time: Res<Time>,
) {
    for (projectile, mut transform, entity) in &mut query {
        // Compute distance vector
        let move_vec = projectile.velocity_vector * time.delta_secs();
        // extend to a Vec3
        let move_vec = move_vec.extend(0.0);
        let loc = &mut transform.translation;
        *loc += move_vec;

        // Despawn the projectiles once they go offscreen.
        let onscreen_x = (-205.0..205.0).contains(&loc.x);
        let onscreen_y = (-405.0..405.0).contains(&loc.y);
        if !(onscreen_x && onscreen_y) {
            commands.entity(entity).despawn();
        }
    }
}
