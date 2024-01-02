use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShowLevelEvent>()
            .add_systems(PreStartup, UiAssets::load)
            .add_systems(Update, (show_level_text, autohide_text));
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
