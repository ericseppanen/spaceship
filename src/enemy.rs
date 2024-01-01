use bevy::prelude::*;

use crate::weapon::{Weapon, WeaponFireEvent};

const ENEMY_SPEED: f32 = 100.0;
const ENEMY_PROJECTILE_VELOCITY: f32 = 400.0;

#[derive(Resource)]
pub struct EnemySpawner {
    /// Enemies spawned per second.
    rate: f32,
    /// Number of enemies to be spawned.
    spawn_remaining: usize,
    /// Time when the next spawn will happen.
    next_spawn: Timer,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            rate: 0.5,
            spawn_remaining: 5,
            // The first spawn is delayed a bit.
            next_spawn: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct Enemy {
    pub velocity: Vec2,
}

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    sprite: SpriteBundle,
    weapon: Weapon,
}

impl EnemyBundle {
    fn new(sprite: SpriteBundle) -> Self {
        let x = if fastrand::bool() {
            ENEMY_SPEED
        } else {
            -ENEMY_SPEED
        };
        let velocity = Vec2 { x, y: ENEMY_SPEED };
        let aim = Vec2 {
            x: 0.0,
            y: ENEMY_PROJECTILE_VELOCITY,
        };
        let weapon = Weapon::new(aim, 0.25);
        Self {
            enemy: Enemy { velocity },
            sprite,
            weapon,
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawner::default())
            .add_systems(Update, (enemy_spawn, enemy_movement));
    }
}

fn enemy_movement(
    mut enemies: Query<(&mut Transform, &mut Enemy)>,
    time: Res<Time>,
    mut _event_sender: EventWriter<WeaponFireEvent>, // FIXME add enemy weapons
) {
    for (mut transform, mut enemy) in &mut enemies {
        let move_vec = enemy.velocity * time.delta_seconds();
        transform.translation += move_vec.extend(0.0);

        // horizontal bounce
        let x = enemy.velocity.x;
        if x > 0.0 {
            if transform.translation.x > 200.0 {
                enemy.velocity.x = -x;
            }
        } else {
            if transform.translation.x < -200.0 {
                enemy.velocity.x = -x;
            }
        }

        // vertical bounce
        let y = enemy.velocity.y;
        if y > 0.0 {
            if transform.translation.y > 400.0 {
                enemy.velocity.y = -y;
            }
        } else {
            if transform.translation.y < -400.0 {
                enemy.velocity.y = -y;
            }
        }
    }
}

fn enemy_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut spawner: ResMut<EnemySpawner>,
) {
    // If the timer has expired, reset it if there are still enemies left to spawn.
    if spawner.next_spawn.just_finished() {
        if spawner.spawn_remaining == 0 {
            // no more spawning to do.
            return;
        }
        let new_timer = Timer::from_seconds(1.0 / spawner.rate, TimerMode::Once);
        spawner.next_spawn = new_timer;
    } else {
        // spawn timer not finished yet.
        spawner.next_spawn.tick(time.delta());
        return;
    }

    // Spawn a new enemy.
    info!("spawn enemy");
    spawner.spawn_remaining -= 1;

    let texture = asset_server.load("green_ship.png");
    let rand = fastrand::f32();
    let x = (rand * 400.0) - 200.0;
    let transform = Transform::from_translation(Vec3::new(x, 410.0, 0.0));
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
    let enemy = EnemyBundle::new(sprite);

    commands.spawn(enemy);
}
