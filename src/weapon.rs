use bevy::prelude::*;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WeaponFireEvent>()
            .add_systems(Update, (charge_weapons, fire_weapon, move_projectiles));
    }
}

#[derive(Component)]
pub struct Projectile {
    pub velocity_vector: Vec2,
    pub owner: Entity,
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    projectile: Projectile,
    sprite: SpriteBundle,
}

#[derive(Component)]
pub struct Weapon {
    ready_timer: Timer,
}

impl Weapon {
    pub fn new(recharge_time: f32) -> Self {
        Self {
            ready_timer: Timer::from_seconds(recharge_time, TimerMode::Once),
        }
    }
}

#[derive(Event)]
pub struct WeaponFireEvent(pub Entity);

/// Handle the `WeaponFireEvent`
fn fire_weapon(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut event: EventReader<WeaponFireEvent>,
    mut query: Query<(&mut Weapon, &Transform)>,
) {
    // Ignore multiple fire events.
    let Some(event) = event.read().next() else {
        return;
    };

    let (mut weapon, transform) = query.get_mut(event.0).expect("missing weapon in entity");

    let timer = &mut weapon.ready_timer;
    if timer.finished() {
        timer.reset();
    } else {
        // weapon is still charging, do nothing.
        return;
    }

    let texture = asset_server.load("green_torpedo.png");
    let sprite = SpriteBundle {
        texture,
        transform: *transform,
        ..default()
    };
    let projectile = Projectile {
        velocity_vector: Vec2 { x: 0.0, y: 400.0 },
        owner: event.0,
    };
    let bundle = ProjectileBundle { projectile, sprite };

    commands.spawn(bundle);
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
        let move_vec = projectile.velocity_vector * time.delta_seconds();
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
