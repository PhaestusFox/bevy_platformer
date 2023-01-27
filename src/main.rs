use bevy::prelude::*;
use rand::Rng;

mod animation;

use animation::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugin(bevy_editor_pls::prelude::EditorPlugin)
    .add_plugin(PhoxAnimationPlugin)
    .add_startup_system(spawn_cam)
    .add_startup_system(spawn_player)
    .add_system(move_player)
    .add_system(player_fall)
    .add_system(player_jump)
    .add_system(ground_detection)
    .add_startup_system(spawn_map)
    .add_system(get_collectable)
    .init_resource::<TerrainSprites>()
    .register_type::<TextureAtlasSprite>()
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
        HitBox(Vec2::new(200., 16.)),
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
        HitBox(Vec2::new(32., 32.)),
    ));
    if let Some((texture_atlas, animation)) = animations.get(Animation::Strawberry) {
        commands.spawn((
            SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(32., 16., 0.)),
                texture_atlas,
                ..Default::default()
            },
            HitBox(Vec2::new(16., 16.)),
            animation,
            FrameTime(0.0),
            Trigger,
            Collectable,
        ));
    }
}

#[derive(Component)]
struct Player;

fn spawn_player(
    mut commands: Commands,
    animations: Res<Animations>,
) {
    let Some((texture_atlas, animation)) = animations.get(Animation::PlayerIdle) else {error!("Failed to find animation: Idle"); return;};
    commands.spawn((SpriteSheetBundle {
        texture_atlas,
        sprite: TextureAtlasSprite {index: 0, ..Default::default()},
        ..Default::default()
    }, Player,
    PhoxAnimationBundle{
        animation,
        frame_time: FrameTime(0.),
    },
    Grounded(true),
    HitBox(Vec2::new(18., 32.)),
    ));
}

const MOVE_SPEED: f32 = 100.;

fn move_player(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Transform, &Grounded, &HitBox), With<Player>>,
    hitboxs: Query<(&HitBox, &Transform), (Without<Player>, Without<Trigger>)>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
) {
    let (entity, mut p_offset, grounded, &p_hitbox) = player.single_mut();
    let delat = if input.any_just_pressed([KeyCode::W, KeyCode::Up, KeyCode::Space]) && grounded.0 {
        commands.entity(entity).insert(Jump(100.));
        return;
    } else if input.any_pressed([KeyCode::A, KeyCode::Left]) {
        -MOVE_SPEED * time.delta_seconds() * (0.5 + (grounded.0 as u16) as f32)
    } else if input.any_pressed([KeyCode::D, KeyCode::Right]) {
        MOVE_SPEED * time.delta_seconds() * (0.5 + (grounded.0 as u16) as f32)
    } else {
        return;
    };
    let new_pos = p_offset.translation + Vec3::X * delat;
    for (&hitbox, offset) in &hitboxs {
        if check_hit(p_hitbox, new_pos, hitbox, offset.translation) {return;}
    }
    p_offset.translation = new_pos;
}

#[derive(Component)]
struct Jump(f32);

const FALL_SPEED: f32 = 98.0;

fn player_jump(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Transform, &mut Jump), With<Player>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((player, mut transform,mut jump)) = player.get_single_mut() else {return;};
    let jump_power = (time.delta_seconds() * FALL_SPEED * 2.).min(jump.0);
    transform.translation.y += jump_power;
    jump.0 -= if input.any_pressed([KeyCode::W, KeyCode::Up, KeyCode::Space]) {jump_power} else {jump_power * 2.};
    if jump.0 <= 0. {
        commands.entity(player).remove::<Jump>();
    }
}

fn player_fall(
    mut player: Query<(&mut Transform, &HitBox), (With<Player>, Without<Jump>)>,
    hitboxs: Query<(&HitBox, &Transform), (Without<Player>, Without<Trigger>)>,
    time: Res<Time>,
) {
    let Ok((mut p_offset, &p_hitbox)) = player.get_single_mut() else {return;};
    let new_pos = p_offset.translation - Vec3::Y * FALL_SPEED * time.delta_seconds();
    for (&hitbox, offset) in &hitboxs {
        if check_hit(p_hitbox, new_pos, hitbox, offset.translation) {return;}
    }
    p_offset.translation = new_pos;
}

#[derive(Component)]
struct Grounded(bool);

fn ground_detection(
    mut player: Query<(&Transform, &mut Grounded), With<Player>>,
    mut last: Local<Transform>,
) {
    let (pos,mut on_ground) = player.single_mut();

    let current = if pos.translation.y == last.translation.y {
        true
    } else {
        false
    };

    if current != on_ground.0 {
        on_ground.0 = current;
    }

    *last = *pos;
}

#[derive(Debug, Component, Clone, Copy)]
struct HitBox(Vec2);

fn check_hit(hitbox: HitBox, offset: Vec3, other_hitbox: HitBox, other_offset: Vec3) -> bool {
    let h_size = hitbox.0.y / 2.;
    let oh_size = other_hitbox.0.y / 2.;
    let w_size = hitbox.0.x / 2.;
    let ow_size = other_hitbox.0.x / 2.;

    offset.x + w_size > other_offset.x - ow_size && offset.x - w_size < other_offset.x + ow_size &&
    offset.y + h_size > other_offset.y - oh_size && offset.y - h_size < other_offset.y + oh_size
}

#[derive(Component)]
struct Trigger;

#[derive(Component)]
struct Collectable;

fn get_collectable(
    player: Query<(&Transform, &HitBox), With<Player>>,
    mut triggers: Query<(&mut Transform, &HitBox), (With<Collectable>, Without<Player>)>,
) {
    let (p_transform, &p_hitbox) = player.single();
    for (mut t_transform, &t_hitbox) in &mut triggers {
        if check_hit(p_hitbox, p_transform.translation, t_hitbox, t_transform.translation) {
            t_transform.translation.x = rand::thread_rng().gen_range(-100.0..100.);
            t_transform.translation.y = rand::thread_rng().gen_range(-10.0..75.);
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