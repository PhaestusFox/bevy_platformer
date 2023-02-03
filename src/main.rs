use bevy::prelude::*;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use rand::Rng;
use leafwing_input_manager::prelude::*;
use bevy_rapier2d::prelude::*;

mod animation;
mod user_input;
mod player;
mod tile_map;

use animation::*;
use player::*;
use tile_map::*;

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
    .insert_resource(RapierConfiguration{
        gravity: Vec2::Y * -294.,
        timestep_mode: TimestepMode::Variable {
            max_dt: 1.0 / 60.0,
            time_scale: 1.0,
            substeps: 10,
        },
        ..Default::default()
    })
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.))
    .add_plugin(RapierDebugRenderPlugin::default())
    .add_plugin(InspectableRapierPlugin)
    .add_plugin(PlayerPlugin)
    .add_plugin(MapPlugin)
    .run()
}

fn spawn_cam(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_map(
    mut commands: Commands,
    animations: Res<Animations>,
    mut map_event: EventWriter<MapEvent>,
) {
    map_event.send(MapEvent::Spawn(Box::new(MapBox{
        offset: IVec3 { x: -6, y: -1, z: 1 },
        width: 13,
        hight: 1,
        material: TerrainMaterial::Gold,
    })));
    map_event.send(MapEvent::Spawn(Box::new(MapBox{
        offset: IVec3 { x: 7, y: 1, z: 1 },
        width: 2,
        hight: 2,
        material: TerrainMaterial::Gold,
    })));
    for i in 0..5 {
        map_event.send(MapEvent::Spawn(Box::new(MapBox{
            offset: IVec3 { x: -7-i, y: i, z: 1 },
            width: 1,
            hight: 1,
            material: TerrainMaterial::Gold,
        })));
    }

    for i in 0..5 {
        map_event.send(MapEvent::Spawn(Box::new(
            MapBox {
                offset: IVec3 { x: i * 2, y: 15, z: 1 },
                width: 1,
                hight: 1,
                material: TerrainMaterial::Brick,
            }
        )));
    }

    map_event.send(MapEvent::Spawn(Box::new(
        MapBox {
            offset: IVec3 { x: -5, y: 10, z: 1 },
            width: 1,
            hight: 4,
            material: TerrainMaterial::Gold,
        }
    )));

    map_event.send(MapEvent::Spawn(Box::new(
        MapBox {
            offset: IVec3 { x: -6, y: 11, z: 1 },
            width: 1,
            hight: 5,
            material: TerrainMaterial::Gold,
        }
    )));

    map_event.send(MapEvent::Spawn(Box::new(
        MapBox {
            offset: IVec3 { x: -10, y: 6, z: 1 },
            width: 2,
            hight: 2,
            material: TerrainMaterial::Gold,
        }
    )));

    if let Some((texture_atlas, animation)) = animations.get(Animation::Strawberry) {
        commands.spawn((
            SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(32., 16., 0.)),
                texture_atlas,
                ..Default::default()
            },
            animation,
            FrameTime(0.0),
            RigidBody::Fixed,
            Collider::ball(8.),
            Sensor,
            Collectable
        ));
    }
}

#[derive(Component)]
struct Collectable;

fn get_collectable(
    player: Query<Entity, With<Player>>,
    mut collectables: Query<&mut Transform, With<Collectable>>,
    rapier_context: Res<RapierContext>,
) {
        let entity = player.single();
    
        /* Iterate through all the intersection pairs involving a specific collider. */
        for (collider1, collider2, intersecting) in rapier_context.intersections_with(entity) {
            if intersecting {
                println!("The entities {:?} and {:?} have intersecting colliders!", collider1, collider2);
                if let Ok(mut pos) = collectables.get_mut(collider2) {
                    pos.translation.x = rand::thread_rng().gen_range(-100.0..100.);
                    pos.translation.y = rand::thread_rng().gen_range(-10.0..150.);
                }
                if let Ok(mut pos) = collectables.get_mut(collider1) {
                    pos.translation.x = rand::thread_rng().gen_range(-100.0..100.);
                    pos.translation.y = rand::thread_rng().gen_range(-10.0..150.);
                }
            }
        }
    }


