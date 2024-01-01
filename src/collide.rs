use bevy::audio::{Volume, VolumeLevel};
use bevy::math::vec2;
use bevy::prelude::*;

use crate::enemy::Enemy;
use crate::player::Player;
use crate::weapon::Projectile;

// FIXME: these hitboxes kind of suck.
// The players and enemies are triangular, and these are rectangles.
const PLAYER_HITBOX: Vec2 = vec2(30.0, 25.0);
const ENEMY_HITBOX: Vec2 = vec2(30.0, 25.0);
const PROJECTILE_HITBOX: Vec2 = vec2(2.0, 4.0);

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDeathEvent>()
            .add_event::<EnemyDeathEvent>()
            .add_systems(Startup, CollisionAssets::load)
            .add_systems(
                Update,
                (
                    check_player_collisions,
                    check_enemy_collisions,
                    player_death,
                    enemy_death,
                    animations,
                ),
            );
    }
}

#[derive(Resource)]
struct CollisionAssets {
    enemy_death_sound: Handle<AudioSource>,
    player_death_sound: Handle<AudioSource>,
    death_animation: Vec<Handle<Image>>,
}

impl CollisionAssets {
    fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
        let enemy_death_sound = asset_server.load("explosion1.wav");
        let player_death_sound = asset_server.load("explosion2.wav");
        let death_animation = vec![
            asset_server.load("explosion1.png"),
            asset_server.load("explosion2.png"),
            asset_server.load("explosion3.png"),
            asset_server.load("explosion4.png"),
            asset_server.load("explosion5.png"),
            asset_server.load("explosion6.png"),
        ];

        commands.insert_resource(CollisionAssets {
            enemy_death_sound,
            player_death_sound,
            death_animation,
        });
    }

    /// Create a bundle to play the enemy death audio.
    fn enemy_death_audio(&self) -> AudioBundle {
        AudioBundle {
            source: self.enemy_death_sound.clone_weak(),
            ..default()
        }
    }

    /// Create a bundle to play the player death audio.
    fn player_death_audio(&self) -> AudioBundle {
        AudioBundle {
            source: self.player_death_sound.clone_weak(),
            settings: PlaybackSettings {
                volume: Volume::Relative(VolumeLevel::new(0.6)),
                ..default()
            },
        }
    }
}

#[derive(Event)]
pub struct PlayerDeathEvent(Entity);

#[derive(Event)]
pub struct EnemyDeathEvent(Entity);

/// Check if the player has collided with something.
///
/// Collisions that we act on:
/// - enemy shots hitting players
/// - player ship hitting enemy ship (destroys both)
///
/// Collisions that are ignored:
/// - player shots hitting players
/// - projectiles hitting other projectiles
///
fn check_player_collisions(
    mut commands: Commands,
    player_query: Query<(&Transform, Entity), With<Player>>,
    projectiles_query: Query<(&Transform, &Projectile, Entity)>,
    enemies_query: Query<(&Transform, Entity), With<Enemy>>,
    mut player_death_sender: EventWriter<PlayerDeathEvent>,
    mut enemy_death_sender: EventWriter<EnemyDeathEvent>,
) {
    use bevy::sprite::collide_aabb::collide;

    let Ok((player_transform, player_entity)) = player_query.get_single() else {
        return;
    };

    // check for player-projectile collisions
    for (projectile_transform, projectile, proj_entity) in &projectiles_query {
        if projectile.player {
            continue;
        }
        if collide(
            projectile_transform.translation,
            PROJECTILE_HITBOX,
            player_transform.translation,
            PLAYER_HITBOX,
        )
        .is_some()
        {
            player_death_sender.send(PlayerDeathEvent(player_entity));
            commands.entity(proj_entity).despawn();
        }
    }

    for (enemy_transform, enemy_entity) in &enemies_query {
        if collide(
            enemy_transform.translation,
            ENEMY_HITBOX,
            player_transform.translation,
            PLAYER_HITBOX,
        )
        .is_some()
        {
            player_death_sender.send(PlayerDeathEvent(player_entity));
            enemy_death_sender.send(EnemyDeathEvent(enemy_entity));
        }
    }
}

/// Check if the enemies have collided with something.
///
/// Collisions that we act on:
/// - player shots hitting enemies
///
/// Collisions that are ignored:
/// - enemy ships hitting each other
/// - enemy shots hitting enemies
/// - projectiles hitting other projectiles
///
fn check_enemy_collisions(
    mut commands: Commands,
    projectiles_query: Query<(&Transform, &Projectile, Entity)>,
    enemies_query: Query<(&Transform, Entity), With<Enemy>>,
    mut enemy_death_sender: EventWriter<EnemyDeathEvent>,
) {
    for (enemy_transform, enemy_entity) in &enemies_query {
        for (projectile_transform, projectile, proj_entity) in &projectiles_query {
            use bevy::sprite::collide_aabb::collide;

            if !projectile.player {
                continue;
            }
            if collide(
                projectile_transform.translation,
                // FIXME: how do I track the projectile size?
                PROJECTILE_HITBOX,
                enemy_transform.translation,
                ENEMY_HITBOX,
            )
            .is_some()
            {
                enemy_death_sender.send(EnemyDeathEvent(enemy_entity));
                commands.entity(proj_entity).despawn();
            }
        }
    }
}

/// Handle player death.
fn player_death(
    mut event: EventReader<PlayerDeathEvent>,
    mut commands: Commands,
    query: Query<&Transform>,
    assets: Res<CollisionAssets>,
) {
    if let Some(event) = event.read().next() {
        info!("player died");
        if let Ok(transform) = query.get(event.0) {
            commands.spawn(DeathAnimation::default().to_bundle(transform, &assets));
        }
        if let Some(mut entity) = commands.get_entity(event.0) {
            entity.despawn();
        };
        commands.spawn(assets.player_death_audio());
    }
}

/// Handle enemy death.
fn enemy_death(
    mut event: EventReader<EnemyDeathEvent>,
    mut commands: Commands,
    query: Query<&Transform>,
    assets: Res<CollisionAssets>,
) {
    for event in event.read() {
        info!("enemy {:?} died", event.0);
        if let Ok(transform) = query.get(event.0) {
            commands.spawn(DeathAnimation::default().to_bundle(transform, &assets));
        }
        if let Some(mut entity) = commands.get_entity(event.0) {
            entity.despawn();
        };
        // FIXME: if the player also died, should we suppress this audio?
        commands.spawn(assets.enemy_death_audio());
    }
}

#[derive(Component)]
struct DeathAnimation {
    index: usize,
    timer: Timer,
}

impl Default for DeathAnimation {
    fn default() -> Self {
        Self {
            index: 0,
            timer: Timer::from_seconds(0.050, TimerMode::Repeating),
        }
    }
}

impl DeathAnimation {
    fn to_bundle(self, transform: &Transform, assets: &CollisionAssets) -> (Self, SpriteBundle) {
        let sprite = SpriteBundle {
            texture: assets.death_animation[0].clone_weak(),
            transform: transform.clone(),
            ..default()
        };
        (self, sprite)
    }
}

/// Play the death animations.
///
/// The entity will be despawned when the animation completes.
fn animations(
    mut commands: Commands,
    mut animation_query: Query<(&mut Handle<Image>, &mut DeathAnimation, Entity)>,
    assets: Res<CollisionAssets>,
    time: Res<Time>,
) {
    for (mut sprite_image, mut animation, entity) in &mut animation_query {
        animation.timer.tick(time.delta());
        if animation.timer.just_finished() {
            animation.index += 1;
            match assets.death_animation.get(animation.index) {
                Some(handle) => {
                    *sprite_image = handle.clone_weak();
                }
                None => {
                    // end of animation.
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}