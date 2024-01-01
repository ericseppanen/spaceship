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
                ),
            );
    }
}

#[derive(Resource)]
struct CollisionAssets {
    enemy_death_sound: Handle<AudioSource>,
    player_death_sound: Handle<AudioSource>,
}

impl CollisionAssets {
    fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
        let enemy_death_sound = asset_server.load("explosion1.wav");
        let player_death_sound = asset_server.load("explosion2.wav");

        commands.insert_resource(CollisionAssets {
            enemy_death_sound,
            player_death_sound,
        });
    }
}

#[derive(Event)]
pub struct PlayerDeathEvent(Entity);

#[derive(Event)]
pub struct EnemyDeathEvent(Entity);

/// Check if something destructible has collided with something else
///
/// Collisions that cause destruction:
/// - player shots hitting enemies
/// - enemy shots hitting players
/// - player ship hitting enemy ship (destroys both)
///
/// Collisions that are ignored:
/// - enemy ship overlapping enemy ship
/// - enemy shots hitting enemies
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

/// Check for collisions between enemy ships and player projectiles
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

// Handle player death
fn player_death(
    mut event: EventReader<PlayerDeathEvent>,
    mut commands: Commands,
    assets: Res<CollisionAssets>,
) {
    if let Some(event) = event.read().next() {
        info!("player died");
        if let Some(mut entity) = commands.get_entity(event.0) {
            entity.despawn();
            commands.spawn(AudioBundle {
                source: assets.player_death_sound.clone_weak(),
                ..default()
            });
        }
    }
}

// Handle enemy death
fn enemy_death(
    mut event: EventReader<EnemyDeathEvent>,
    mut commands: Commands,
    assets: Res<CollisionAssets>,
) {
    for event in event.read() {
        info!("enemy {:?} died", event.0);
        if let Some(mut entity) = commands.get_entity(event.0) {
            entity.despawn();
            commands.spawn(AudioBundle {
                source: assets.enemy_death_sound.clone_weak(),
                ..default()
            });
        }
    }
}
