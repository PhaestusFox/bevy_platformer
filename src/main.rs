use bevy::prelude::*;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use rand::Rng;
use leafwing_input_manager::prelude::*;
use bevy_rapier2d::prelude::*;

mod animation;
mod user_input;
mod player;

use animation::*;
use player::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugin(bevy_editor_pls::prelude::EditorPlugin)
    .add_plugin(PhoxAnimationPlugin)
    .add_startup_system(spawn_cam)
    .add_startup_system(spawn_map)
    .add_system(get_collectable)
    .init_resource::<TerrainSprites>()
    .register_type::<TextureAtlasSprite>()
    .add_plugin(InputManagerPlugin::<user_input::PlayerInput>::default())
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.))
    .add_plugin(RapierDebugRenderPlugin::default())
    .add_plugin(InspectableRapierPlugin)
    .add_plugin(PlayerPlugin)
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
    terrain: Res<TerrainSprites>,
) {
    commands.spawn((
        SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::NEG_Y * 32.),
            sprite: TextureAtlasSprite { custom_size: Some(Vec2::new(168., 16.)),
                color: Color::WHITE,
                index: TerrainType::GoldStright as usize,
                ..Default::default()
            },
            texture_atlas: terrain.get_atlas(),
            ..Default::default()
        },
        RigidBody::Fixed,
        Collider::cuboid(100., 8.),
    )).with_children(|p| {
        p.spawn(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::X * 92.),
            sprite: TextureAtlasSprite { custom_size: Some(Vec2::new(16., 16.)),
                color: Color::WHITE,
                index: TerrainType::GoldRightEnd as usize,
                ..Default::default()
            },
            texture_atlas: terrain.get_atlas(),
            ..Default::default()
        });
        p.spawn(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::NEG_X * 92.),
            sprite: TextureAtlasSprite { custom_size: Some(Vec2::new(16., 16.)),
                color: Color::WHITE,
                index: TerrainType::GoldLeftEnd as usize,
                ..Default::default()
            },
            texture_atlas: terrain.get_atlas(),
            ..Default::default()
        });
    });
    commands.spawn((
        SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(110., 20., 0.)),
            sprite: TextureAtlasSprite { custom_size: Some(Vec2::new(32., 32.)),
                color: Color::WHITE,
                index: TerrainType::GoldLeftEnd as usize,
                ..Default::default()
            },
            texture_atlas: terrain.get_atlas(),
            ..Default::default()
        },
        RigidBody::Fixed,
        Collider::cuboid(16., 16.),
    ));
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

#[derive(Resource)]
struct TerrainSprites(Handle<TextureAtlas>);

impl TerrainSprites {
    fn new(handle: Handle<TextureAtlas>) -> TerrainSprites {
        TerrainSprites(handle)
    }
    fn get_atlas(&self) -> Handle<TextureAtlas> {
        self.0.clone()
    }
}

impl FromWorld for TerrainSprites {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture_atles = TextureAtlas::from_grid(asset_server.load("Terrain/Terrain (16x16).png"), Vec2::splat(16.), 22, 11, None, None);
        let mut assets = world.resource_mut::<Assets<TextureAtlas>>();
        TerrainSprites::new(assets.add(texture_atles))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TerrainType {
    GoldLeftEnd = 193,
    GoldStright = 194,
    GoldRightEnd = 195,
}
