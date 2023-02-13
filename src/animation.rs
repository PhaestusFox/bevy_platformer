use std::collections::HashMap;

use bevy::prelude::*;

use super::player::*;
use super::*;

pub struct PhoxAnimationPlugin;

impl Plugin for PhoxAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sprite)
            .add_system(change_player_animation)
            .init_resource::<Animations>();
    }
}

#[derive(Component, Clone, Copy)]
pub struct SpriteAnimation {
    pub len: usize,
    pub frame_time: f32,
}

impl SpriteAnimation {
    fn new(len: usize, fps: usize) -> SpriteAnimation {
        SpriteAnimation {
            len,
            frame_time: 1. / fps as f32,
        }
    }
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

#[derive(Bundle)]
pub struct PhoxAnimationBundle {
    pub animaiton: SpriteAnimation,
    frame_time: FrameTime,
}

impl PhoxAnimationBundle {
    pub fn new(animation: SpriteAnimation) -> PhoxAnimationBundle {
        PhoxAnimationBundle {
            animaiton: animation,
            frame_time: FrameTime(Timer::new(std::time::Duration::from_secs_f32(animation.frame_time), TimerMode::Repeating)),
        }
    }
}

fn animate_sprite(
    mut animations: Query<(&mut TextureAtlasSprite, &SpriteAnimation, &mut FrameTime)>,
    time: Res<Time>,
) {
    for (mut sprite, animation, mut frame_time) in animations.iter_mut() {
        frame_time.tick(time.delta());
        sprite.index += frame_time.frames();
        if sprite.index >= animation.len {
            sprite.index %= animation.len;
        }
    }
}

#[derive(Resource)]
pub struct Animations {
    map: HashMap<Animation, (Handle<TextureAtlas>, SpriteAnimation)>,
}

impl FromWorld for Animations {
    fn from_world(world: &mut World) -> Self {
        let mut map = Animations {
            map: HashMap::new(),
        };
        world.resource_scope(|world, mut texture_atles: Mut<Assets<TextureAtlas>>| {
            let asset_server = world.resource::<AssetServer>();
            // Mask Dude
            let idel_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Idle (32x32).png"),
                Vec2::splat(32.),
                11,
                1,
                None,
                None,
            );
            map.add(
                Animation::MaskIdle,
                texture_atles.add(idel_atlas),
                SpriteAnimation::new(11, 20),
            );

            let run_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Run (32x32).png"),
                Vec2::splat(32.),
                12,
                1,
                None,
                None,
            );
            map.add(
                Animation::MaskRun,
                texture_atles.add(run_atlas),
                SpriteAnimation::new(12, 20),
            );

            let jump_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Jump (32x32).png"),
                Vec2::splat(32.),
                1,
                1,
                None,
                None,
            );
            map.add(
                Animation::MaskJump,
                texture_atles.add(jump_atlas),
                SpriteAnimation::new(1, 1),
            );

            let djump_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Double Jump (32x32).png"),
                Vec2::splat(32.),
                6,
                1,
                None,
                None,
            );
            map.add(
                Animation::MaskDubbleJump,
                texture_atles.add(djump_atlas),
                SpriteAnimation::new(6, 20),
            );

            let fall_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Mask Dude/Fall (32x32).png"),
                Vec2::splat(32.),
                1,
                1,
                None,
                None,
            );
            map.add(
                Animation::MaskFall,
                texture_atles.add(fall_atlas),
                SpriteAnimation::new(1, 1),
            );

            // Collectables
            let strawberry_atlas = TextureAtlas::from_grid(
                asset_server.load("Items/Fruits/Strawberry.png"),
                Vec2::splat(32.),
                17,
                1,
                None,
                None,
            );
            map.add(
                Animation::Strawberry,
                texture_atles.add(strawberry_atlas),
                SpriteAnimation::new(17, 20),
            );

            // Ninja Frog
            let idel_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Ninja Frog/Idle (32x32).png"),
                Vec2::splat(32.),
                11,
                1,
                None,
                None,
            );
            map.add(
                Animation::NinjaIdle,
                texture_atles.add(idel_atlas),
                SpriteAnimation::new(11, 20),
            );

            let run_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Ninja Frog/Run (32x32).png"),
                Vec2::splat(32.),
                12,
                1,
                None,
                None,
            );
            map.add(
                Animation::NinjaRun,
                texture_atles.add(run_atlas),
                SpriteAnimation::new(12, 20),
            );

            let jump_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Ninja Frog/Jump (32x32).png"),
                Vec2::splat(32.),
                1,
                1,
                None,
                None,
            );
            map.add(
                Animation::NinjaJump,
                texture_atles.add(jump_atlas),
                SpriteAnimation::new(1, 1),
            );

            let djump_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Ninja Frog/Double Jump (32x32).png"),
                Vec2::splat(32.),
                6,
                1,
                None,
                None,
            );
            map.add(
                Animation::NinjaDubbleJump,
                texture_atles.add(djump_atlas),
                SpriteAnimation::new(6, 20),
            );

            let fall_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Ninja Frog/Fall (32x32).png"),
                Vec2::splat(32.),
                1,
                1,
                None,
                None,
            );
            map.add(
                Animation::NinjaFall,
                texture_atles.add(fall_atlas),
                SpriteAnimation::new(1, 1),
            );

            //Pink Man
            let idel_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Pink Man/Idle (32x32).png"),
                Vec2::splat(32.),
                11,
                1,
                None,
                None,
            );
            map.add(
                Animation::PinkIdle,
                texture_atles.add(idel_atlas),
                SpriteAnimation::new(11, 20),
            );

            let run_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Pink Man/Run (32x32).png"),
                Vec2::splat(32.),
                12,
                1,
                None,
                None,
            );
            map.add(
                Animation::PinkRun,
                texture_atles.add(run_atlas),
                SpriteAnimation::new(12, 20),
            );

            let jump_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Pink Man/Jump (32x32).png"),
                Vec2::splat(32.),
                1,
                1,
                None,
                None,
            );
            map.add(
                Animation::PinkJump,
                texture_atles.add(jump_atlas),
                SpriteAnimation::new(1, 1),
            );

            let djump_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Pink Man/Double Jump (32x32).png"),
                Vec2::splat(32.),
                6,
                1,
                None,
                None,
            );
            map.add(
                Animation::PinkDubbleJump,
                texture_atles.add(djump_atlas),
                SpriteAnimation::new(6, 20),
            );

            let fall_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Pink Man/Fall (32x32).png"),
                Vec2::splat(32.),
                1,
                1,
                None,
                None,
            );
            map.add(
                Animation::PinkFall,
                texture_atles.add(fall_atlas),
                SpriteAnimation::new(1, 1),
            );
            // Virtual Guy
            let idel_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Virtual Guy/Idle (32x32).png"),
                Vec2::splat(32.),
                11,
                1,
                None,
                None,
            );
            map.add(
                Animation::GuyIdle,
                texture_atles.add(idel_atlas),
                SpriteAnimation::new(11, 20),
            );

            let run_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Virtual Guy/Run (32x32).png"),
                Vec2::splat(32.),
                12,
                1,
                None,
                None,
            );
            map.add(
                Animation::GuyRun,
                texture_atles.add(run_atlas),
                SpriteAnimation::new(12, 20),
            );

            let jump_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Virtual Guy/Jump (32x32).png"),
                Vec2::splat(32.),
                1,
                1,
                None,
                None,
            );
            map.add(
                Animation::GuyJump,
                texture_atles.add(jump_atlas),
                SpriteAnimation::new(1, 1),
            );

            let djump_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Virtual Guy/Double Jump (32x32).png"),
                Vec2::splat(32.),
                6,
                1,
                None,
                None,
            );
            map.add(
                Animation::GuyDubbleJump,
                texture_atles.add(djump_atlas),
                SpriteAnimation::new(6, 20),
            );

            let fall_atlas = TextureAtlas::from_grid(
                asset_server.load("Main Characters/Virtual Guy/Fall (32x32).png"),
                Vec2::splat(32.),
                1,
                1,
                None,
                None,
            );
            map.add(
                Animation::GuyFall,
                texture_atles.add(fall_atlas),
                SpriteAnimation::new(1, 1),
            );
        });

        map
    }
}

impl Animations {
    pub fn add(&mut self, id: Animation, handle: Handle<TextureAtlas>, animation: SpriteAnimation) {
        self.map.insert(id, (handle, animation));
    }
    pub fn get(&self, id: Animation) -> Option<(Handle<TextureAtlas>, SpriteAnimation)> {
        self.map.get(&id).cloned()
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Animation {
    MaskRun,
    MaskIdle,
    MaskJump,
    MaskDubbleJump,
    MaskFall,
    Strawberry,
    NinjaRun,
    NinjaIdle,
    NinjaJump,
    NinjaDubbleJump,
    NinjaFall,
    PinkRun,
    PinkIdle,
    PinkJump,
    PinkDubbleJump,
    PinkFall,
    GuyRun,
    GuyIdle,
    GuyJump,
    GuyDubbleJump,
    GuyFall,
}

fn change_player_animation(
    mut player: Query<
        (
            &Player,
            &mut Handle<TextureAtlas>,
            &mut SpriteAnimation,
            &mut TextureAtlasSprite,
            &Jump,
            &Velocity,
        )
    >,
    animaitons: Res<Animations>,
) {
    for (player, mut atlas, mut animation, mut sprite, jump, velocity) in &mut player {
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
            Animation::MaskDubbleJump
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
            Animation::MaskDubbleJump => Animation::NinjaDubbleJump,
            Animation::MaskFall => Animation::NinjaFall,
            _ => unreachable!(),
        },
        Player::Pink => match set {
            Animation::MaskRun => Animation::PinkRun,
            Animation::MaskIdle => Animation::PinkIdle,
            Animation::MaskJump => Animation::PinkJump,
            Animation::MaskDubbleJump => Animation::PinkDubbleJump,
            Animation::MaskFall => Animation::PinkFall,
            _ => unreachable!(),
        },
        Player::Guy => match set {
            Animation::MaskRun => Animation::GuyRun,
            Animation::MaskIdle => Animation::GuyIdle,
            Animation::MaskJump => Animation::GuyJump,
            Animation::MaskDubbleJump => Animation::GuyDubbleJump,
            Animation::MaskFall => Animation::GuyFall,
            _ => unreachable!(),
        },
    };

    let Some((new_atlas, new_animaiton)) = animaitons.get(set) else {error!("No Animation Jump Loaded"); return;};
    *atlas = new_atlas;
    sprite.index %= new_animaiton.len;
    *animation = new_animaiton;
    }
}
