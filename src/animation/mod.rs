use std::collections::HashMap;

use bevy::prelude::*;
use bevy::reflect::TypeUuid;

use super::player::*;
use super::*;

mod loader;

pub struct PhoxAnimationPlugin;

impl Plugin for PhoxAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sprite)
            .add_system(change_player_animation)
            .add_system(update_animation_components)
            .add_system(add_frame_time)
            .add_asset::<SpriteAnimation>()
            .add_asset_loader(loader::AnimationLoader)
            .init_resource::<Animations>()
            .init_asset_loader::<loader::AnimationLoader>();
    }
}

#[derive(TypeUuid)]
#[uuid = "5b68f25a-835d-45f2-855d-94613a2da2fd"]
pub struct SpriteAnimation {
    pub len: usize,
    pub frame_time: f32,
    pub texture_atlas: Handle<TextureAtlas>,
}

#[derive(Component)]
pub struct FrameTime(pub Timer);

impl FrameTime {
    fn tick(&mut self, delta: std::time::Duration) {
        self.0.tick(delta);
    }
    fn finished(&self) -> bool {
        self.0.finished()
    }
    fn frames(&self) -> usize {
        self.0.times_finished_this_tick() as usize
    }
}

fn animate_sprite(
    mut entitys: Query<(
        &mut TextureAtlasSprite,
        &Handle<SpriteAnimation>,
        &mut FrameTime,
    )>,
    animaitons: Res<Assets<SpriteAnimation>>,
    time: Res<Time>,
) {
    for (mut sprite, animation, mut frame_time) in entitys.iter_mut() {
        let Some(animation) = animaitons.get(animation) else {error!("Animation Not Loaded"); continue;};
        frame_time.tick(time.delta());
        sprite.index += frame_time.frames();
        if sprite.index >= animation.len {
            sprite.index %= animation.len;
        }
    }
}

#[derive(Resource)]
pub struct Animations {
    map: HashMap<Animation, Handle<SpriteAnimation>>,
}

impl FromWorld for Animations {
    fn from_world(world: &mut World) -> Self {
        let mut map = Animations {
            map: HashMap::new(),
        };
        let asset_server = world.resource::<AssetServer>();
        // Mask Dude
        map.add(
            Animation::MaskIdle,
            asset_server.load("Animations/Mask.san.ron"),
        );
        map.add(
            Animation::MaskFall,
            asset_server.load("Animations/Mask.san.ron#Fall"),
        );
        map.add(
            Animation::MaskJump,
            asset_server.load("Animations/Mask.san.ron#Jump"),
        );
        map.add(
            Animation::MaskDoubleJump,
            asset_server.load("Animations/Mask.san.ron#DoubleJump"),
        );
        map.add(
            Animation::MaskRun,
            asset_server.load("Animations/Mask.san.ron#Run"),
        );

        //Pink Man
        map.add(
            Animation::PinkIdle,
            asset_server.load("Animations/Pink.san.ron"),
        );
        map.add(
            Animation::PinkFall,
            asset_server.load("Animations/Pink.san.ron#Fall"),
        );
        map.add(
            Animation::PinkJump,
            asset_server.load("Animations/Pink.san.ron#Jump"),
        );
        map.add(
            Animation::PinkDoubleJump,
            asset_server.load("Animations/Pink.san.ron#DoubleJump"),
        );
        map.add(
            Animation::PinkRun,
            asset_server.load("Animations/Pink.san.ron#Run"),
        );

        //Ninja Frog
        map.add(
            Animation::NinjaIdle,
            asset_server.load("Animations/Ninja.san.ron"),
        );
        map.add(
            Animation::NinjaFall,
            asset_server.load("Animations/Ninja.san.ron#Fall"),
        );
        map.add(
            Animation::NinjaJump,
            asset_server.load("Animations/Ninja.san.ron#Jump"),
        );
        map.add(
            Animation::NinjaDoubleJump,
            asset_server.load("Animations/Ninja.san.ron#DoubleJump"),
        );
        map.add(
            Animation::NinjaRun,
            asset_server.load("Animations/Ninja.san.ron#Run"),
        );

        //Virtual Guy
        map.add(
            Animation::GuyIdle,
            asset_server.load("Animations/Guy.san.ron"),
        );
        map.add(
            Animation::GuyFall,
            asset_server.load("Animations/Guy.san.ron#Fall"),
        );
        map.add(
            Animation::GuyJump,
            asset_server.load("Animations/Guy.san.ron#Jump"),
        );
        map.add(
            Animation::GuyDoubleJump,
            asset_server.load("Animations/Guy.san.ron#DoubleJump"),
        );
        map.add(
            Animation::GuyRun,
            asset_server.load("Animations/Guy.san.ron#Run"),
        );

        // Collectables
        map.add(
            Animation::Strawberry,
            asset_server.load("Animations/Collectables.san.ron#Strawberry"),
        );

        map
    }
}

impl Animations {
    pub fn add(&mut self, id: Animation, handle: Handle<SpriteAnimation>) {
        self.map.insert(id, handle);
    }
    pub fn get(&self, id: Animation) -> Option<Handle<SpriteAnimation>> {
        self.map.get(&id).cloned()
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Animation {
    MaskRun,
    MaskIdle,
    MaskJump,
    MaskDoubleJump,
    MaskFall,
    Strawberry,
    NinjaRun,
    NinjaIdle,
    NinjaJump,
    NinjaDoubleJump,
    NinjaFall,
    PinkRun,
    PinkIdle,
    PinkJump,
    PinkDoubleJump,
    PinkFall,
    GuyRun,
    GuyIdle,
    GuyJump,
    GuyDoubleJump,
    GuyFall,
}

fn change_player_animation(
    mut player: Query<(
        &Player,
        &mut Handle<SpriteAnimation>,
        &mut TextureAtlasSprite,
        &Jump,
        &Velocity,
    )>,
    animaitons: Res<Animations>,
) {
    for (player, mut animation, mut sprite, jump, velocity) in &mut player {
        if velocity.linvel.x < -0.1 {
            sprite.flip_x = true;
        } else if velocity.linvel.x > 0.1 {
            sprite.flip_x = false;
        }

        //Jumping if jump
        let set = if velocity.linvel.y > 0.01 {
            if jump.0 {
                Animation::MaskJump
            } else {
                Animation::MaskDoubleJump
            }
        //Falling if no on ground
        } else if velocity.linvel.y < -0.01 {
            Animation::MaskFall
        // if any move keys pressed set run sprite
        } else if velocity.linvel.x != 0.0 {
            Animation::MaskRun
        } else {
            Animation::MaskIdle
        };

        let set = match player {
            Player::Mask => set,
            Player::Ninja => match set {
                Animation::MaskRun => Animation::NinjaRun,
                Animation::MaskIdle => Animation::NinjaIdle,
                Animation::MaskJump => Animation::NinjaJump,
                Animation::MaskDoubleJump => Animation::NinjaDoubleJump,
                Animation::MaskFall => Animation::NinjaFall,
                _ => unreachable!(),
            },
            Player::Pink => match set {
                Animation::MaskRun => Animation::PinkRun,
                Animation::MaskIdle => Animation::PinkIdle,
                Animation::MaskJump => Animation::PinkJump,
                Animation::MaskDoubleJump => Animation::PinkDoubleJump,
                Animation::MaskFall => Animation::PinkFall,
                _ => unreachable!(),
            },
            Player::Guy => match set {
                Animation::MaskRun => Animation::GuyRun,
                Animation::MaskIdle => Animation::GuyIdle,
                Animation::MaskJump => Animation::GuyJump,
                Animation::MaskDoubleJump => Animation::GuyDoubleJump,
                Animation::MaskFall => Animation::GuyFall,
                _ => unreachable!(),
            },
        };

        let Some(handle) = animaitons.get(set) else {error!("No Animation {:?} Loaded", set); return;};
        *animation = handle;
    }
}

fn update_animation_components(
    mut query: Query<
        (
            &Handle<SpriteAnimation>,
            &mut FrameTime,
            &mut Handle<TextureAtlas>,
            &mut TextureAtlasSprite,
        ),
        Changed<Handle<SpriteAnimation>>,
    >,
    animations: Res<Assets<SpriteAnimation>>,
) {
    for (animation_handle, mut frame_time, mut atlas, mut sprite) in &mut query {
        let Some(animation) = animations.get(animation_handle) else {error!("animation not found"); continue;};
        frame_time
            .0
            .set_duration(std::time::Duration::from_secs_f32(animation.frame_time));
        *atlas = animation.texture_atlas.clone();
        sprite.index %= animation.len;
    }
}

fn add_frame_time(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Handle<SpriteAnimation>,
            &mut Handle<TextureAtlas>,
            &mut TextureAtlasSprite,
        ),
        (Added<Handle<SpriteAnimation>>, Without<FrameTime>),
    >,
    animations: Res<Assets<SpriteAnimation>>,
) {
    for (entity, animation_handle, mut atlas, mut sprite) in &mut query {
        let Some(animation) = animations.get(animation_handle) else {error!("animation not found"); continue;};
        commands.entity(entity).insert(FrameTime(Timer::new(
            std::time::Duration::from_secs_f32(animation.frame_time),
            TimerMode::Repeating,
        )));
        *atlas = animation.texture_atlas.clone();
        sprite.index %= animation.len;
    }
}
