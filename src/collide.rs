use bevy::audio::{PlaybackMode, Volume};
use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::math::vec2;
use bevy::prelude::*;

use crate::enemy::Enemy;
use crate::level::LevelRestartEvent;
use crate::player::Player;
use crate::ui::{GameOverEvent, PlayerLives};
use crate::weapon::Projectile;
use crate::GameState;

// FIXME: these hitboxes kind of suck.
// The players and enemies are triangular, and these are rectangles.
const PLAYER_HITBOX: Vec2 = vec2(15.0, 12.5);
const ENEMY_HITBOX: Vec2 = vec2(15.0, 12.5);
const PROJECTILE_HITBOX: Vec2 = vec2(1.0, 2.0);

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDeathEvent>()
            .add_event::<EnemyDeathEvent>()
            .add_systems(Startup, CollisionAssets::load)
            .add_systems(
                Update,
                (check_player_collisions, check_enemy_collisions)
                    .run_if(in_state(GameState::Playing)),
            )
            // Make sure the player death runs in the same frame as the
            // collision was detected; otherwise the collision could be
            // detected twice.
            .add_systems(
                Update,
                player_death
                    .after(check_player_collisions)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                enemy_death
                    .after(check_player_collisions)
                    .after(check_enemy_collisions)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, death_animations);
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
    fn enemy_death_audio(&self, commands: &mut Commands) {
        commands.spawn((
            AudioPlayer(self.enemy_death_sound.clone_weak()),
            PlaybackSettings::DESPAWN,
        ));
    }

    /// Create a bundle to play the player death audio.
    fn player_death_audio(&self, commands: &mut Commands) {
        commands.spawn((
            AudioPlayer(self.player_death_sound.clone_weak()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::new(0.6),
                ..default()
            },
        ));
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
    let Ok((player_transform, player_entity)) = player_query.get_single() else {
        return;
    };

    // check for player-projectile collisions
    for (projectile_transform, projectile, proj_entity) in &projectiles_query {
        if projectile.player {
            continue;
        }

        let projectile_box = Aabb2d::new(
            projectile_transform.translation.truncate(),
            PROJECTILE_HITBOX,
        );
        let player_box = Aabb2d::new(player_transform.translation.truncate(), PLAYER_HITBOX);
        if projectile_box.intersects(&player_box) {
            player_death_sender.send(PlayerDeathEvent(player_entity));
            commands.entity(proj_entity).despawn();
        }
    }

    // check for player-enemy collisions
    for (enemy_transform, enemy_entity) in &enemies_query {
        let enemy_box = Aabb2d::new(enemy_transform.translation.truncate(), ENEMY_HITBOX);
        let player_box = Aabb2d::new(player_transform.translation.truncate(), PLAYER_HITBOX);
        if enemy_box.intersects(&player_box) {
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
            if !projectile.player {
                continue;
            }

            let projectile_box = Aabb2d::new(
                projectile_transform.translation.truncate(),
                PROJECTILE_HITBOX,
            );
            let enemy_box = Aabb2d::new(enemy_transform.translation.truncate(), ENEMY_HITBOX);
            if projectile_box.intersects(&enemy_box) {
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
    mut level_reset: EventWriter<LevelRestartEvent>,
    mut game_over: EventWriter<GameOverEvent>,
    mut lives: ResMut<PlayerLives>,
) {
    // Consume all events (which should be equivalent), keeping the last.
    if let Some(event) = event.read().last() {
        info!("player died");
        if let Ok(transform) = query.get(event.0) {
            commands.spawn(DeathAnimation::default().into_bundle(transform, &assets));
        }
        if let Some(mut entity) = commands.get_entity(event.0) {
            entity.despawn();
        };

        assets.player_death_audio(&mut commands);
        **lives = lives.checked_sub(1).unwrap();
        if **lives == 0 {
            game_over.send(GameOverEvent);
        }
        // LevelRestartEvent despawns all the enemies, so
        // we should do this even if the game is over.
        level_reset.send(LevelRestartEvent);
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
            commands.spawn(DeathAnimation::default().into_bundle(transform, &assets));
        }
        if let Some(mut entity) = commands.get_entity(event.0) {
            entity.despawn();
        };
        // FIXME: if the player also died, should we suppress this audio?
        assets.enemy_death_audio(&mut commands);
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
    fn into_bundle(
        self,
        transform: &Transform,
        assets: &CollisionAssets,
    ) -> (Self, Sprite, Transform) {
        let sprite = Sprite::from_image(assets.death_animation[0].clone_weak());
        (self, sprite, *transform)
    }
}

/// Play the death animations.
///
/// The entity will be despawned when the animation completes.
fn death_animations(
    mut commands: Commands,
    mut animation_query: Query<(&mut Sprite, &mut DeathAnimation, Entity)>,
    assets: Res<CollisionAssets>,
    time: Res<Time>,
) {
    for (mut sprite, mut animation, entity) in &mut animation_query {
        animation.timer.tick(time.delta());
        if animation.timer.just_finished() {
            animation.index += 1;
            match assets.death_animation.get(animation.index) {
                Some(handle) => {
                    sprite.image = handle.clone_weak();
                }
                None => {
                    // end of animation.
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
