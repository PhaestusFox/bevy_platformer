use std::collections::HashMap;

use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};

use super::player::*;
use super::*;

mod loader;

pub struct PhoxAnimationPlugin;

impl Plugin for PhoxAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sprite)
            .add_systems(Update, change_player_animation)
            .add_systems(Update, update_animation_components)
            .add_systems(Last, add_frame_time)
            .add_asset::<SpriteAnimation>()
            .add_asset_loader(loader::AnimationLoader)
            .init_resource::<Animations>()
            .init_asset_loader::<loader::AnimationLoader>();
    }
}

#[derive(TypeUuid, TypePath)]
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
    mut entities: Query<(
        &mut TextureAtlasSprite,
        &Handle<SpriteAnimation>,
        &mut FrameTime,
    )>,
    animations: Res<Assets<SpriteAnimation>>,
    time: Res<Time>,
) {
    for (mut sprite, animation, mut frame_time) in entities.iter_mut() {
        let Some(animation) = animations.get(animation) else {error!("Animation Not Loaded"); continue;};
        frame_time.tick(time.delta());
        sprite.index += frame_time.frames();
        sprite.index %= animation.len;
    }
}

#[derive(Resource)]
pub struct Animations {
    animations: HashMap<Animation, Handle<SpriteAnimation>>,
    atlas: HashMap<Animation, Handle<TextureAtlas>>,
}

impl FromWorld for Animations {
    fn from_world(world: &mut World) -> Self {
        let mut map = Animations {
            animations: HashMap::new(),
            atlas: HashMap::new(),
        };
        let asset_server = world.resource::<AssetServer>();
        // Mask Dude
        map.add_animation(
            Animation::MaskIdle,
            asset_server.load("Animations/Mask.san.ron"),
        );
        map.add_animation(
            Animation::MaskFall,
            asset_server.load("Animations/Mask.san.ron#Fall"),
        );
        map.add_animation(
            Animation::MaskJump,
            asset_server.load("Animations/Mask.san.ron#Jump"),
        );
        map.add_animation(
            Animation::MaskDoubleJump,
            asset_server.load("Animations/Mask.san.ron#DoubleJump"),
        );
        map.add_animation(
            Animation::MaskRun,
            asset_server.load("Animations/Mask.san.ron#Run"),
        );

        //Pink Man
        map.add_animation(
            Animation::PinkIdle,
            asset_server.load("Animations/Pink.san.ron"),
        );
        map.add_animation(
            Animation::PinkFall,
            asset_server.load("Animations/Pink.san.ron#Fall"),
        );
        map.add_animation(
            Animation::PinkJump,
            asset_server.load("Animations/Pink.san.ron#Jump"),
        );
        map.add_animation(
            Animation::PinkDoubleJump,
            asset_server.load("Animations/Pink.san.ron#DoubleJump"),
        );
        map.add_animation(
            Animation::PinkRun,
            asset_server.load("Animations/Pink.san.ron#Run"),
        );

        //Ninja Frog
        map.add_animation(
            Animation::NinjaIdle,
            asset_server.load("Animations/Ninja.san.ron"),
        );
        map.add_animation(
            Animation::NinjaFall,
            asset_server.load("Animations/Ninja.san.ron#Fall"),
        );
        map.add_animation(
            Animation::NinjaJump,
            asset_server.load("Animations/Ninja.san.ron#Jump"),
        );
        map.add_animation(
            Animation::NinjaDoubleJump,
            asset_server.load("Animations/Ninja.san.ron#DoubleJump"),
        );
        map.add_animation(
            Animation::NinjaRun,
            asset_server.load("Animations/Ninja.san.ron#Run"),
        );

        //Virtual Guy
        map.add_animation(
            Animation::GuyIdle,
            asset_server.load("Animations/Guy.san.ron"),
        );
        map.add_animation(
            Animation::GuyFall,
            asset_server.load("Animations/Guy.san.ron#Fall"),
        );
        map.add_animation(
            Animation::GuyJump,
            asset_server.load("Animations/Guy.san.ron#Jump"),
        );
        map.add_animation(
            Animation::GuyDoubleJump,
            asset_server.load("Animations/Guy.san.ron#DoubleJump"),
        );
        map.add_animation(
            Animation::GuyRun,
            asset_server.load("Animations/Guy.san.ron#Run"),
        );

        // Collectables
        map.add_animation(
            Animation::Strawberry,
            asset_server.load("Animations/Collectables.san.ron#Strawberry"),
        );
        map.add_animation(
            Animation::Bananas,
            asset_server.load("Animations/Collectables.san.ron#Bananas"),
        );

        //terrain
        map.add_atlas(
            Animation::Terrain,
            asset_server.load("Animations/Terrain.san.ron#Atlas"),
        );

        map
    }
}

impl Animations {
    pub fn add_animation(&mut self, id: Animation, handle: Handle<SpriteAnimation>) {
        self.animations.insert(id, handle);
    }
    pub fn get_animation(&self, id: Animation) -> Option<Handle<SpriteAnimation>> {
        self.animations.get(&id).cloned()
    }
    pub fn get_atlas(&self, id: Animation) -> Option<Handle<TextureAtlas>> {
        self.atlas.get(&id).cloned()
    }
    pub fn add_atlas(&mut self, id: Animation, handle: Handle<TextureAtlas>) {
        self.atlas.insert(id, handle);
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
    Bananas,
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
    Terrain,
}

fn change_player_animation(
    mut player: Query<(
        &Player,
        &mut Handle<SpriteAnimation>,
        &mut TextureAtlasSprite,
        &Jump,
        &Velocity,
    )>,
    animations: Res<Animations>,
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

        let Some(handle) = animations.get_animation(set) else {error!("No Animation {:?} Loaded", set); return;};
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
