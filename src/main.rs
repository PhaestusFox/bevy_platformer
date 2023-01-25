use std::collections::HashMap;

use bevy::prelude::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(bevy_editor_pls::prelude::EditorPlugin)
    .add_startup_system(spawn_cam)
    .add_startup_system(spawn_player)
    .add_system(animate_sprite)
    .add_system(move_player)
    .add_system(change_player_animation)
    .init_resource::<Animations>()
    .add_system(player_fall)
    .add_system(player_jump)
    .add_system(ground_detection)
    .add_startup_system(spawn_map)
    .add_system(get_collectable)
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
) {
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::NEG_Y * 16.),
            sprite: Sprite { custom_size: Some(Vec2::new(200., 5.)),
                color: Color::WHITE,
                ..Default::default()
            },
            ..Default::default()
        },
        HitBox(Vec2::new(200., 5.)),
    ));
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(100., 25., 0.)),
            sprite: Sprite { custom_size: Some(Vec2::new(32., 32.)),
                color: Color::WHITE,
                ..Default::default()
            },
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
            HitBox(Vec2::new(32., 32.)),
            animation,
            FrameTime(0.0),
            Trigger,
        ));
    }
}

#[derive(Component)]
struct Player;

fn spawn_player(
    mut commands: Commands,
    animaitons: Res<Animations>,
) {
    let Some((texture_atlas, animation)) = animaitons.get(Animation::PlayerIdle) else {error!("Failed to find animation: Idle"); return;};
    commands.spawn((SpriteSheetBundle {
        texture_atlas,
        sprite: TextureAtlasSprite {index: 0, ..Default::default()},
        ..Default::default()
    }, Player,
    animation,
    FrameTime(0.0),
    Grounded(true),
    HitBox(Vec2::splat(32.)),
    ));
}

#[derive(Component, Clone, Copy)]
struct SpriteAnimation {
    len: usize,
    frame_time: f32,
}

#[derive(Component)]
struct FrameTime(f32);

fn animate_sprite(
    mut animations: Query<(&mut TextureAtlasSprite, &SpriteAnimation, &mut FrameTime)>,
    time: Res<Time>,
) {
    for (mut sprite, animation, mut frame_time) in animations.iter_mut() {
        frame_time.0 += time.delta_seconds();
        if frame_time.0 > animation.frame_time {
            let frames = (frame_time.0 / animation.frame_time) as usize;
            sprite.index += frames;
            if sprite.index >= animation.len {
                sprite.index %= animation.len;
            }
            frame_time.0 -= animation.frame_time;
        }
    }
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

fn change_player_animation(
    mut player: Query<(&mut Handle<TextureAtlas>, &mut SpriteAnimation, &mut TextureAtlasSprite), With<Player>>,
    player_jump: Query<(Option<&Jump>, &Grounded), With<Player>>,
    input: Res<Input<KeyCode>>,
    animaitons: Res<Animations>,
) {
    let (mut atlas, mut animation, mut sprite) = player.single_mut();
    let (jump, grounded) = player_jump.single();
    if input.any_just_pressed([KeyCode::A, KeyCode::Left]) {
        sprite.flip_x = true;
    } else if input.any_just_pressed([KeyCode::D, KeyCode::Right])
    && !input.any_pressed([KeyCode::A, KeyCode::Left]) {
        sprite.flip_x = false;
    } else if input.any_just_released([KeyCode::A, KeyCode::Left])
    && !input.any_pressed([KeyCode::A, KeyCode::Left])
    && input.any_pressed([KeyCode::D, KeyCode::Right]) {
        sprite.flip_x = false;
    }
    
    let set = 
    //Jumping if jump
    if jump.is_some() {
        Animation::PlayerJump
    //Falling if no on ground
    } else if !grounded.0 {
        Animation::PlayerFall
    // if any move keys pressed set run sprite
    } else if input.any_pressed([KeyCode::A, KeyCode::Left, KeyCode::D, KeyCode::Right]) {
        Animation::PlayerRun
    } else {
        Animation::PlayerIdle
    };

    let Some((new_atlas, new_animaiton)) = animaitons.get(set) else {error!("No Animation Jump Loaded"); return;};
    *atlas = new_atlas;
    sprite.index %= new_animaiton.len;
    *animation = new_animaiton;
}

#[derive(Resource)]
struct Animations {
    map: HashMap<Animation, (Handle<TextureAtlas>, SpriteAnimation)>,
}

impl FromWorld for Animations {
    fn from_world(world: &mut World) -> Self {
        let mut map = Animations {map: HashMap::new()};
        world.resource_scope(|world, mut texture_atles: Mut<Assets<TextureAtlas>>| {
            let asset_server = world.resource::<AssetServer>();
            let idel_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Idle (32x32).png"),
                Vec2::splat(32.),
                11, 1, None, None);
            map.add(Animation::PlayerIdle, texture_atles.add(idel_atlas), SpriteAnimation { len: 11, frame_time: 1./20. });
            
            let run_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Run (32x32).png"),
                Vec2::splat(32.),
                12, 1, None, None);
            map.add(Animation::PlayerRun, texture_atles.add(run_atlas), SpriteAnimation { len: 12, frame_time: 1./20. });
            
            let jump_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Jump (32x32).png"),
                Vec2::splat(32.),
                1, 1, None, None);
            map.add(Animation::PlayerJump, texture_atles.add(jump_atlas), SpriteAnimation { len: 1, frame_time: 1. });
            
            let fall_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Fall (32x32).png"),
                Vec2::splat(32.),
                1, 1, None, None);
            map.add(Animation::PlayerFall, texture_atles.add(fall_atlas), SpriteAnimation { len: 1, frame_time: 1. });
                
            let strawberry_atlas = TextureAtlas::from_grid(
                asset_server.load("Items/Fruits/Strawberry.png"),
                Vec2::splat(32.),
                17, 1, None, None);
            map.add(Animation::Strawberry, texture_atles.add(strawberry_atlas), SpriteAnimation { len: 17, frame_time: 1./20. });
        });

        map
    }
}

impl Animations {
    fn add(&mut self, id: Animation, handle: Handle<TextureAtlas>, animation: SpriteAnimation) {
        self.map.insert(id, (handle, animation));
    }
    fn get(&self, id: Animation) -> Option<(Handle<TextureAtlas>, SpriteAnimation)> {
        self.map.get(&id).cloned()
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Animation {
    PlayerRun,
    PlayerIdle,
    PlayerJump,
    PlayerFall,
    Strawberry,
}

#[derive(Component)]
struct Jump(f32);

const FALL_SPEED: f32 = 98.0;

fn player_jump(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Transform, &mut Jump), With<Player>>,
    time: Res<Time>,
) {
    let Ok((player, mut transform,mut jump)) = player.get_single_mut() else {return;};
    let jump_power = (time.delta_seconds() * FALL_SPEED * 2.).min(jump.0);
    jump.0 -= jump_power;
    transform.translation.y += jump_power;
    if jump.0 == 0. {
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

fn get_collectable(
    player: Query<(&Transform, &HitBox), With<Player>>,
    triggers: Query<(Entity, &Transform, &HitBox), (With<Trigger>, Without<Player>)>,
    mut commands: Commands,
) {
    let (p_transform, &p_hitbox) = player.single();
    for (entity, t_transform, &t_hitbox) in &triggers {
        if check_hit(p_hitbox, p_transform.translation, t_hitbox, t_transform.translation) {
            commands.entity(entity).despawn();
        }
    }
}