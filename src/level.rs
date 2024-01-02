use std::cmp::min;

use bevy::prelude::*;

use crate::enemy::SpawnerResetEvent;
use crate::player::PlayerSpawnEvent;
use crate::ui::ShowLevelEvent;

#[derive(Debug, Clone)]
pub struct Level {
    pub enemy_speed: f32,
    pub num_scout: usize,
    pub num_fighter: usize,
    pub spawn_rate: f32,
}

const LEVELS: &[Level] = &[
    Level {
        enemy_speed: 90.0,
        num_scout: 5,
        num_fighter: 1,
        spawn_rate: 0.5,
    },
    Level {
        enemy_speed: 100.0,
        num_scout: 8,
        num_fighter: 2,
        spawn_rate: 1.0,
    },
    Level {
        enemy_speed: 110.0,
        num_scout: 12,
        num_fighter: 3,
        spawn_rate: 1.5,
    },
];

#[derive(Resource)]
pub struct CurrentLevel {
    pub number: usize,
    pub level: Level,
    pub level_start_timer: Timer,
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self {
            number: 0,
            level: LEVELS[0].clone(),
            level_start_timer: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentLevel::default())
            .add_event::<LevelEndEvent>()
            .add_event::<LevelRestartEvent>()
            .add_systems(Update, (level_start, detect_player_death, bump_level));
    }
}

/// The player has completed the level.
#[derive(Event)]
pub struct LevelEndEvent;

/// The player died; restart the level.
#[derive(Event)]
pub struct LevelRestartEvent;

fn level_start(
    time: Res<Time>,
    mut current_level: ResMut<CurrentLevel>,
    mut spawn_player: EventWriter<PlayerSpawnEvent>,
    mut enemies: EventWriter<SpawnerResetEvent>,
    mut level_text: EventWriter<ShowLevelEvent>,
) {
    current_level.level_start_timer.tick(time.delta());

    if current_level.level_start_timer.just_finished() {
        info!("start level {}", current_level.number);

        level_text.send(ShowLevelEvent(format!("LEVEL {}", current_level.number)));

        spawn_player.send(PlayerSpawnEvent);
        enemies.send(SpawnerResetEvent(current_level.level.clone()));
    }
}

/// If the player died, trigger a level restart.
fn detect_player_death(
    mut current_level: ResMut<CurrentLevel>,
    mut event: EventReader<LevelRestartEvent>,
) {
    if let Some(_) = event.read().last() {
        current_level.level_start_timer.reset();
    };
}

/// If the player completed the level, load the next one.
fn bump_level(mut current_level: ResMut<CurrentLevel>, mut event: EventReader<LevelEndEvent>) {
    if let Some(_) = event.read().last() {
        let level_number = current_level.number + 1;
        let level_index = min(level_number, LEVELS.len() - 1);
        let mut level = LEVELS[level_index].clone();
        if level_number >= 3 {
            level.enemy_speed += (level_number - 2) as f32 * 15.0;
        }
        *current_level = CurrentLevel {
            number: level_number,
            level,
            level_start_timer: Timer::from_seconds(2.0, TimerMode::Once),
        };
    };
}
