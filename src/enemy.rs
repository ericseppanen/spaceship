use std::collections::VecDeque;

use bevy::math::vec2;
use bevy::prelude::*;

use crate::collide::EnemyDeathEvent;
use crate::level::{Level, LevelEndEvent, LevelRestartEvent};
use crate::ui::Score;
use crate::weapon::{Weapon, WeaponFireEvent};
use crate::GameState;

const ENEMY_PROJECTILE_VELOCITY: f32 = 400.0;

#[derive(Resource)]
struct EnemyAssets {
    scout_image: Handle<Image>,
    fighter_image: Handle<Image>,
}

impl EnemyAssets {
    fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
        let scout_image = asset_server.load("green_ship.png");
        let fighter_image = asset_server.load("blue_ship.png");

        commands.insert_resource(EnemyAssets {
            scout_image,
            fighter_image,
        });
    }
}

#[derive(Debug, Default, Resource)]
pub struct EnemySpawner {
    /// Enemies spawned per second.
    rate: f32,
    /// Enemies waiting to be spawned.
    spawn_queue: VecDeque<Enemy>,
    /// Number of enemies to kill before the level ends.
    level_remaining: usize,
    /// Time when the next spawn will happen.
    next_spawn: Timer,
}

impl From<&Level> for EnemySpawner {
    fn from(level: &Level) -> Self {
        let mut spawn_queue = VecDeque::with_capacity(level.num_scout + level.num_fighter);
        // How many scouts spawn between fighters.
        let fighter_cadence = level.num_scout / level.num_fighter;
        for count in 0..level.num_scout {
            spawn_queue.push_back(Enemy::Scout {
                speed: level.enemy_speed,
            });

            if (1 + count) % fighter_cadence == 0 {
                spawn_queue.push_back(Enemy::Fighter {
                    speed: level.enemy_speed,
                })
            }
        }

        let this = Self {
            rate: level.spawn_rate,
            spawn_queue,
            level_remaining: level.num_scout + level.num_fighter,
            next_spawn: Timer::from_seconds(3.0, TimerMode::Once),
        };
        info!("{this:?}");
        this
    }
}

#[derive(Debug, Component)]
pub enum Enemy {
    Scout { speed: f32 },
    Fighter { speed: f32 },
}

#[derive(Debug, Component)]
pub enum MovementPattern {
    Zigzag(Vec2),
    LeftRight(f32),
}

#[derive(Component)]
struct WeaponBehavior {
    timer: Timer,
}

impl WeaponBehavior {}

pub struct EnemyBundle {
    enemy: Enemy,
    movement: MovementPattern,
    sprite: SpriteBundle,
    weapon: Option<Weapon>,
    weapon_behavior: Option<WeaponBehavior>,
}

impl EnemyBundle {
    fn spawn(self, commands: &mut Commands) {
        let mut commands = commands.spawn((self.enemy, self.movement, self.sprite));
        if let Some(weapon) = self.weapon {
            commands.insert(weapon);
        }
        if let Some(weapon_behavior) = self.weapon_behavior {
            commands.insert(weapon_behavior);
        }
    }
}

impl Enemy {
    fn make_bundle(self, assets: &EnemyAssets) -> EnemyBundle {
        let movement = match &self {
            Enemy::Scout { speed } => {
                let x = if fastrand::bool() { *speed } else { -*speed };
                MovementPattern::Zigzag(vec2(x, *speed))
            }
            Enemy::Fighter { speed } => {
                let x = if fastrand::bool() { *speed } else { -*speed };
                MovementPattern::LeftRight(x)
            }
        };

        let texture = match &self {
            Enemy::Scout { .. } => assets.scout_image.clone_weak(),
            Enemy::Fighter { .. } => assets.fighter_image.clone_weak(),
        };
        let rand = fastrand::f32();
        let x = (rand * 400.0) - 200.0;

        let spawn_point = match &self {
            Enemy::Scout { .. } => vec2(x, 410.0),
            Enemy::Fighter { .. } => vec2(x, 370.0),
        };

        let transform = Transform::from_translation(spawn_point.extend(0.0));
        let sprite = Sprite {
            flip_y: true,
            ..default()
        };
        let sprite = SpriteBundle {
            sprite,
            texture,
            transform,
            ..default()
        };

        let aim = vec2(0.0, -ENEMY_PROJECTILE_VELOCITY);
        let (weapon, weapon_behavior) = match &self {
            Enemy::Scout { .. } => (None, None),
            Enemy::Fighter { .. } => (
                Some(Weapon::new(aim, 0.25)),
                Some(WeaponBehavior {
                    timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                }),
            ),
        };
        EnemyBundle {
            enemy: self,
            movement,
            sprite,
            weapon,
            weapon_behavior,
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnerResetEvent>()
            .insert_resource(EnemySpawner::default())
            .add_systems(Startup, EnemyAssets::load)
            .add_systems(
                Update,
                (
                    enemy_spawn,
                    enemy_movement,
                    enemy_weapons,
                    level_restart_despawn,
                    reset_spawner,
                    enemy_death,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn enemy_movement(
    mut enemies: Query<(&mut Transform, &mut MovementPattern), With<Enemy>>,
    time: Res<Time<Virtual>>,
) {
    for (mut transform, mut movement) in &mut enemies {
        match &mut *movement {
            MovementPattern::LeftRight(x_velocity) => {
                let move_x = *x_velocity * time.delta_seconds();
                transform.translation.x += move_x;

                if *x_velocity > 0.0 {
                    if transform.translation.x > 200.0 {
                        *x_velocity = -*x_velocity;
                    }
                } else if transform.translation.x < -200.0 {
                    *x_velocity = -*x_velocity;
                }
            }
            MovementPattern::Zigzag(velocity) => {
                let move_vec = *velocity * time.delta_seconds();
                transform.translation += move_vec.extend(0.0);

                // horizontal bounce
                if velocity.x > 0.0 {
                    if transform.translation.x > 200.0 {
                        velocity.x = -velocity.x;
                    }
                } else if transform.translation.x < -200.0 {
                    velocity.x = -velocity.x;
                }

                // vertical bounce
                if velocity.y > 0.0 {
                    if transform.translation.y > 400.0 {
                        velocity.y = -velocity.y;
                    }
                } else if transform.translation.y < -400.0 {
                    velocity.y = -velocity.y;
                }
            }
        }
    }
}

fn enemy_weapons(
    time: Res<Time<Virtual>>,
    mut enemies: Query<(&mut WeaponBehavior, Entity), With<Enemy>>,
    mut event_sender: EventWriter<WeaponFireEvent>,
) {
    for (mut behavior, entity) in &mut enemies {
        behavior.timer.tick(time.delta());
        if behavior.timer.just_finished() {
            event_sender.send(WeaponFireEvent(entity));
        }
    }
}

fn enemy_spawn(
    mut commands: Commands,
    assets: Res<EnemyAssets>,
    time: Res<Time<Virtual>>,
    mut spawner: ResMut<EnemySpawner>,
) {
    let enemy;
    // If the timer has expired, reset it if there are still enemies left to spawn.
    if spawner.next_spawn.just_finished() {
        match spawner.spawn_queue.pop_front() {
            Some(e) => enemy = e,
            None => {
                // no more spawning to do.
                return;
            }
        };
        let new_timer = Timer::from_seconds(1.0 / spawner.rate, TimerMode::Once);
        spawner.next_spawn = new_timer;
    } else {
        // spawn timer not finished yet.
        spawner.next_spawn.tick(time.delta());
        return;
    }

    // Spawn a new enemy.
    info!("spawn enemy {enemy:?}");
    enemy.make_bundle(&assets).spawn(&mut commands);
}

/// Load level settings and reset the spawner.
#[derive(Event)]
pub struct SpawnerResetEvent(pub Level);

fn reset_spawner(mut event: EventReader<SpawnerResetEvent>, mut spawner: ResMut<EnemySpawner>) {
    let Some(SpawnerResetEvent(level)) = event.read().last() else {
        return;
    };
    *spawner = EnemySpawner::from(level);
}

/// Despawn all enemies due to `LevelRestartEvent`
fn level_restart_despawn(
    mut commands: Commands,
    mut event: EventReader<LevelRestartEvent>,
    mut spawner: ResMut<EnemySpawner>,
    enemies: Query<Entity, With<Enemy>>,
) {
    let Some(_) = event.read().last() else {
        return;
    };
    // Disable the timer so no new enemies will spawn.
    spawner.next_spawn.pause();

    for entity in &enemies {
        commands.entity(entity).despawn();
    }
}

/// Count dead enemies and bump the level on the last one.
fn enemy_death(
    mut event: EventReader<EnemyDeathEvent>,
    mut spawner: ResMut<EnemySpawner>,
    mut level_end: EventWriter<LevelEndEvent>,
    mut score: ResMut<Score>,
) {
    for _event in event.read() {
        score.0 += 100;
        spawner.level_remaining = spawner.level_remaining.checked_sub(1).unwrap();
        if spawner.level_remaining == 0 {
            level_end.send(LevelEndEvent);
        }
    }
}
