use bevy::prelude::*;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier2d::prelude::*;
use ghost::GhostEvents;
use leafwing_input_manager::prelude::*;

mod animation;
mod ghost;
mod player;
mod user_input;
mod map;

use animation::*;
use player::*;
use map::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(bevy_editor_pls::prelude::EditorPlugin)
        .add_plugin(PhoxAnimationPlugin)
        .add_startup_system(spawn_cam)
        .add_startup_system(spawn_map)
        .add_system(get_collectable)
        .register_type::<TextureAtlasSprite>()
        .add_plugin(InputManagerPlugin::<user_input::PlayerInput>::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::Y * -294.,
            timestep_mode: TimestepMode::Fixed {
                dt: 1. / 60.,
                substeps: 1,
            },
            ..Default::default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InspectableRapierPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(ghost::GhostPlugin)
        .insert_resource(Score(0))
        .run()
}

fn spawn_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_map(
    mut map_event: EventWriter<MapEvent>,
) {
    map_event.send(MapEvent::spawn(MapBox {
        offset: IVec3 { x: -6, y: -1, z: 1 },
        width: 13,
        hight: 1,
        material: TerrainMaterial::Gold,
    }));
    map_event.send(MapEvent::Spawn(Box::new(MapBox {
        offset: IVec3 { x: 7, y: 1, z: 1 },
        width: 2,
        hight: 2,
        material: TerrainMaterial::Gold,
    })));
    map_event.send(MapEvent::Spawn(Box::new(MapBox {
        offset: IVec3 { x: 7, y: 1, z: 1 },
        width: 1,
        hight: 1,
        material: TerrainMaterial::Clay,
    })));
    for i in 0..5 {
        map_event.send(MapEvent::Spawn(Box::new(MapBox {
            offset: IVec3 {
                x: -11,
                y: 4-i,
                z: 1,
            },
            width: 1 + i,
            hight: 1,
            material: TerrainMaterial::Gold,
        })));
    }

    for i in 0..5 {
        map_event.send(MapEvent::Spawn(Box::new(MapBox {
            offset: IVec3 {
                x: i * 2,
                y: 15,
                z: 1,
            },
            width: 1,
            hight: 1,
            material: TerrainMaterial::Brick,
        })));
    }

    map_event.send(MapEvent::Spawn(Box::new(MapBox {
        offset: IVec3 { x: -5, y: 10, z: 1 },
        width: 1,
        hight: 4,
        material: TerrainMaterial::Gold,
    })));

    map_event.send(MapEvent::Spawn(Box::new(MapBox {
        offset: IVec3 { x: -6, y: 9, z: 1 },
        width: 1,
        hight: 5,
        material: TerrainMaterial::Gold,
    })));
    map_event.send(MapEvent::Spawn(Box::new(MapBox {
        offset: IVec3 { x: -6, y: 9, z: 1 },
        width: 1,
        hight: 1,
        material: TerrainMaterial::Clay,
    })));

    map_event.send(MapEvent::Spawn(Box::new(MapBox {
        offset: IVec3 { x: -10, y: 6, z: 1 },
        width: 2,
        hight: 2,
        material: TerrainMaterial::Gold,
    })));

    map_event.send(MapEvent::Spawn(Box::new(MapBox {
        offset: IVec3 { x: -2, y: 7, z: 1 },
        width: 5,
        hight: 1,
        material: TerrainMaterial::Copper,
    })));

    map_event.send(MapEvent::Spawn(Box::new(MapBox {
        offset: IVec3 { x: -2, y: 8, z: 1 },
        width: 4,
        hight: 1,
        material: TerrainMaterial::Iron,
    })));

    map_event.send(MapEvent::spawn(Collectable {
        collectable_type: CollectableType::Strawberry,
        spawn_type: SpawnType::Fixed(IVec2::new(2, 1)),
    }));

    map_event.send(MapEvent::spawn(Collectable {
        collectable_type: CollectableType::Bananan,
        spawn_type: SpawnType::RandomRange(IVec2::new(-10, 0), IVec2::new(10, 20)),
    }));
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
                map_events.send(MapEvent::spawn(collectable.clone()));
                score.0 += 1;
                commands.entity(collider2).despawn_recursive();
            }
            if let Ok(collectable) = collectables.get_mut(collider1) {
                map_events.send(MapEvent::spawn(collectable.clone()));
                events.send(GhostEvents::SpawnGhost);
                score.0 += 1;
                commands.entity(collider2).despawn_recursive();
            }
        }
    }
}

#[derive(Resource)]
struct Score(usize);
