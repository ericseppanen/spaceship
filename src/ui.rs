use std::ops::{Deref, DerefMut};

use bevy::math::vec3;
use bevy::prelude::*;
use bevy::winit::WinitWindows;

use crate::level::CurrentLevel;
use crate::{scancodes, GameState};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShowLevelEvent>()
            .add_event::<GameOverEvent>()
            .add_state::<IconState>()
            .insert_resource(Score::default())
            .insert_resource(PlayerLives::default())
            .add_systems(PreStartup, UiAssets::load)
            .add_systems(Startup, (create_score, create_intro_text))
            .add_systems(
                Update,
                set_window_icon.run_if(in_state(IconState::NotLoaded)),
            )
            .add_systems(Update, start_game.run_if(in_state(GameState::Idle)))
            .add_systems(
                Update,
                (
                    show_level_text,
                    autohide_text,
                    update_score,
                    pause_game,
                    game_over,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource)]
struct UiAssets {
    font: Handle<Font>,
}

impl UiAssets {
    fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
        let font = asset_server.load("monoMMM_5.ttf");
        commands.insert_resource(UiAssets { font });
    }
}

#[derive(Default, Resource)]
pub struct PlayerLives(pub usize);

impl Deref for PlayerLives {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlayerLives {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LevelText;

/// Either the start of game message or the "game over" message.
#[derive(Component)]
pub struct InterstitialText;

#[derive(Component)]
pub struct AutoHide {
    fade: Timer,
}

impl Default for AutoHide {
    fn default() -> Self {
        Self {
            fade: Timer::from_seconds(2.5, TimerMode::Once),
        }
    }
}

fn create_intro_text(assets: Res<UiAssets>, commands: Commands) {
    create_interstitial_text("FIRE TO START", assets, commands)
}

fn create_gameover_text(assets: Res<UiAssets>, commands: Commands) {
    create_interstitial_text("GAME OVER", assets, commands)
}

fn create_interstitial_text(text: &str, assets: Res<UiAssets>, mut commands: Commands) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font: assets.font.clone_weak(),
                    font_size: 20.0,
                    ..default()
                },
            )
            .with_alignment(TextAlignment::Center),
            ..default()
        },
        InterstitialText,
    ));
}

#[derive(Event)]
pub struct ShowLevelEvent(pub String);

fn show_level_text(
    assets: Res<UiAssets>,
    mut commands: Commands,
    mut event: EventReader<ShowLevelEvent>,
) {
    let Some(ShowLevelEvent(level_text)) = event.read().last() else {
        return;
    };

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                level_text,
                TextStyle {
                    font: assets.font.clone_weak(),
                    font_size: 20.0,
                    ..default()
                },
            )
            .with_alignment(TextAlignment::Center),
            ..default()
        },
        AutoHide::default(),
        LevelText,
    ));
}

fn autohide_text(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut AutoHide, Entity)>,
    mut commands: Commands,
) {
    for (mut text, mut autohide, entity) in &mut query {
        autohide.fade.tick(time.delta());
        let alpha = 2.0 * autohide.fade.percent_left();
        if alpha > 1.0 {
            return;
        }
        for section in &mut text.sections {
            section.style.color.set_a(alpha);
        }
        if autohide.fade.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Default, Resource)]
pub struct Score(pub u32);

fn create_score(assets: Res<UiAssets>, mut commands: Commands) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "------",
                TextStyle {
                    font: assets.font.clone_weak(),
                    font_size: 20.0,
                    ..default()
                },
            )
            .with_alignment(TextAlignment::Center),
            transform: Transform::from_translation(vec3(150.0, 380.0, -1.0)),
            ..default()
        },
        ScoreText,
    ));
}

fn update_score(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    use std::fmt::Write;

    let mut score_text = query.single_mut();
    let score_string = &mut score_text.sections[0].value;
    score_string.clear();
    write!(score_string, "{:06}", score.0).unwrap();
}

fn pause_game(
    mut time: ResMut<Time<Virtual>>,
    keyboard: Res<Input<ScanCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(scancodes::ESC) {
        if time.is_paused() {
            time.unpause();
            next_state.set(GameState::Playing);
        } else {
            time.pause();
            next_state.set(GameState::Paused);
        }
    }
}

fn start_game(
    keyboard: Res<Input<ScanCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut lives: ResMut<PlayerLives>,
    mut score: ResMut<Score>,
    mut current_level: ResMut<CurrentLevel>,
    start_text: Query<Entity, With<InterstitialText>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(scancodes::SPACE) {
        info!("start game");
        next_state.set(GameState::Playing);
        lives.0 = 3;
        score.0 = 0;
        // FIXME: is there a better way to do this?
        *current_level = default();

        // If the interstitial text is still onscreen make it go away.
        if let Ok(entity) = start_text.get_single() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Event)]
pub struct GameOverEvent;

fn game_over(
    mut event: EventReader<GameOverEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    assets: Res<UiAssets>,
    commands: Commands,
) {
    let Some(_) = event.read().last() else {
        return;
    };

    info!("game over");
    create_gameover_text(assets, commands);

    next_state.set(GameState::Idle);
}

/// Tracks whether we set the window icon.
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, States)]
enum IconState {
    #[default]
    NotLoaded,
    Loaded,
}

fn set_window_icon(
    windows: NonSend<WinitWindows>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    mut next_state: ResMut<NextState<IconState>>,
) {
    use winit::window::Icon;

    let player_ship_image: Handle<Image> = asset_server.load("red_ship.png");
    let Some(image) = images.get(&player_ship_image) else {
        return;
    };
    let size = image.texture_descriptor.size;
    let width = size.width;
    let height = size.height;

    let icon = Icon::from_rgba(image.data.clone(), width, height).unwrap();

    // do it for all windows
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
    info!("loaded icon");
    next_state.set(IconState::Loaded);
}
