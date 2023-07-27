use bevy::prelude::*;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier2d::prelude::*;
use ghost::GhostEvents;
use leafwing_input_manager::prelude::*;

mod animation;
mod editor;
mod ghost;
mod map;
mod menu;
mod player;
mod user_input;

use animation::*;
use map::*;
use player::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(bevy_editor_pls::prelude::EditorPlugin::default())
        .add_plugins(PhoxAnimationPlugin)
        .add_systems(Startup, spawn_cam)
        .add_systems(Startup, spawn_map)
        .add_systems(Update, get_collectable)
        .register_type::<TextureAtlasSprite>()
        .add_plugins(InputManagerPlugin::<user_input::PlayerInput>::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::Y * -294.,
            timestep_mode: TimestepMode::Fixed {
                dt: 1. / 60.,
                substeps: 1,
            },
            ..Default::default()
        })
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(PlayerPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(ghost::GhostPlugin)
        .insert_resource(Score(0))
        .add_state::<GameState>()
        .add_plugins(menu::MenuPlugin)
        .add_plugins(editor::LevelEditorPlugin)
        .run()
}

#[derive(Component)]
struct MainCam;

fn spawn_cam(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCam));
}

#[derive(Debug, Resource)]
struct CurrentLevel(Handle<Level>, bool);

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LoadedLevel(asset_server.load("Levels/test.lvl.ron")));
}

fn get_collectable(
    mut commands: Commands,
    player: Query<Entity, With<RealPlayer>>,
    mut collectables: Query<&Collectable>,
    rapier_context: Res<RapierContext>,
    mut events: EventWriter<GhostEvents>,
    mut map_events: EventWriter<MapEvent>,
    mut score: ResMut<Score>,
) {
    let entity = player.single();
    /* Iterate through all the intersection pairs involving a specific collider. */
    for (collider1, collider2, intersecting) in rapier_context.intersections_with(entity) {
        if intersecting {
            if let Ok(collectable) = collectables.get_mut(collider2) {
                events.send(GhostEvents::SpawnGhost);
                map_events.send(MapEvent::spawn(Clone::clone(collectable)));
                score.0 += 1;
                commands.entity(collider2).despawn_recursive();
            }
            if let Ok(collectable) = collectables.get_mut(collider1) {
                map_events.send(MapEvent::spawn(Clone::clone(collectable)));
                events.send(GhostEvents::SpawnGhost);
                score.0 += 1;
                commands.entity(collider2).despawn_recursive();
            }
        }
    }
}

#[derive(Resource)]
struct Score(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, States, Default)]
enum GameState {
    Play,
    #[default]
    Menu,
    InputLevelBase64,
    InputLevelName,
    LevelEditor,
}
