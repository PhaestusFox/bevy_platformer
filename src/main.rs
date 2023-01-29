use bevy::prelude::*;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use rand::Rng;
use leafwing_input_manager::prelude::*;
use bevy_rapier2d::prelude::*;

mod animation;
mod user_input;

use animation::*;
use user_input::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugin(bevy_editor_pls::prelude::EditorPlugin)
    .add_plugin(PhoxAnimationPlugin)
    .add_startup_system(spawn_cam)
    .add_startup_system(spawn_player)
    .add_system(move_player)
    .add_system(ground_detection)
    .add_startup_system(spawn_map)
    .add_system(get_collectable)
    .add_system(dubble_jump.before(move_player))
    .add_system(change_player)
    .init_resource::<TerrainSprites>()
    .register_type::<TextureAtlasSprite>()
    .add_plugin(InputManagerPlugin::<user_input::PlayerInput>::default())
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.))
    .add_plugin(InspectableRapierPlugin)
    .register_type::<Grounded>()
    .register_type::<Jump>()
    .register_type::<Player>()
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

#[derive(Component, Reflect, PartialEq)]
enum Player {
    Mask,
    Ninja,
    Pink,
    Guy,
}

fn spawn_player(
    mut commands: Commands,
    animations: Res<Animations>,
) {
    let Some((texture_atlas, animation)) = animations.get(Animation::MaskIdle) else {error!("Failed to find animation: Idle"); return;};
    commands.spawn((SpriteSheetBundle {
        texture_atlas,
        sprite: TextureAtlasSprite {index: 0, ..Default::default()},
        ..Default::default()
    },
    Player::Mask,
    PhoxAnimationBundle{
        animation,
        frame_time: FrameTime(0.),
    },
    Grounded(true),
    InputManagerBundle {
        input_map: PlayerInput::player_one(),
        ..Default::default()
    },
    Jump(false),
    RigidBody::Dynamic,
    Velocity::default(),
    Collider::cuboid(9., 16.),
    LockedAxes::ROTATION_LOCKED_Z,
    Name::new("Player"),
    ));
}

const MOVE_SPEED: f32 = 100.;

fn move_player(
    mut player: Query<(&mut Velocity, &ActionState<PlayerInput>, &Grounded), With<Player>>,
) {
    let (mut velocity, input, grounded) = player.single_mut();
    if input.just_pressed(PlayerInput::Jump) && grounded.0 {
        velocity.linvel.y = 100.;
    } else if input.just_pressed(PlayerInput::Fall) {
        velocity.linvel.y = velocity.linvel.y.min(0.0);
    } else if input.pressed(PlayerInput::Left) {
        velocity.linvel.x = -MOVE_SPEED;
    } else if input.pressed(PlayerInput::Right) {
        velocity.linvel.x = MOVE_SPEED;
    } else if input.just_released(PlayerInput::Left) {
        velocity.linvel.x = 0.0;
    } else if input.just_released(PlayerInput::Right) {
        velocity.linvel.x = 0.0;
    };
}

fn dubble_jump(
    mut player: Query<(&mut Jump, &mut Velocity, &ActionState<PlayerInput>), With<Player>>,
    can_jump: Query<(Entity, &Grounded), Changed<Grounded>>,
) {
    for (entity, grounded) in &can_jump {
        if let Ok((mut jump, _, _)) = player.get_mut(entity) {
            if grounded.0 {
                jump.0 = true;
            }
        }
    }
    for (mut jump, mut velocity, input) in player.iter_mut() {
        if velocity.linvel.y.abs() < 0.01 {return;}
        if input.just_pressed(PlayerInput::Jump) && jump.0 {
            jump.0 = false;
            velocity.linvel.y = 100.;
        }
    }
}

fn change_player(
    mut query: Query<(&mut Player, &ActionState<PlayerInput>)>
) {
    for (mut player, state) in &mut query {
        if state.just_pressed(PlayerInput::NextPlayer) {
            *player = match *player {
                Player::Mask => Player::Ninja,
                Player::Ninja => Player::Pink,
                Player::Pink => Player::Guy,
                Player::Guy => Player::Mask,
            };
        } else if state.just_pressed(PlayerInput::PevPlayer) {
            *player = match *player {
                Player::Mask => Player::Ninja,
                Player::Ninja => Player::Pink,
                Player::Pink => Player::Guy,
                Player::Guy => Player::Mask,
            };
        }
    }
}

#[derive(Component, Reflect)]
struct Jump(bool);

#[derive(Component, Reflect)]
struct Grounded(bool);

fn ground_detection(
    mut player: Query<(&Transform, &mut Grounded), With<Player>>,
    mut last: Local<(f32, isize)>,
) {
    let (pos,mut on_ground) = player.single_mut();

    if (pos.translation.y * 100.).round() == last.0 {
        last.1 += 1;
    } else {
        last.1 -= 1;
    };
    last.1 = last.1.clamp(0, 3);

    if last.1 == 3 && !on_ground.0 {
        on_ground.0 = true;
    } else if last.1 == 0 && on_ground.0 {
        on_ground.0 = false;
    }

    last.0 = (pos.translation.y * 100.).round();
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