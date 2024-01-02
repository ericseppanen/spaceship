use bevy::math::vec3;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShowLevelEvent>()
            .insert_resource(Score(0))
            .add_systems(PreStartup, UiAssets::load)
            .add_systems(Startup, create_score)
            .add_systems(Update, (show_level_text, autohide_text, update_score));
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

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LevelText;

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
            info!("despawn level text");
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Resource)]
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
