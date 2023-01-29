use bevy::prelude::*;
use crate::animation::{Animation, Animations, PhoxAnimationBundle};
use crate::user_input::PlayerInput;
use leafwing_input_manager::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_startup_system(spawn_player)
        .add_system(move_player)
        .add_system(ground_detection)
        .add_system(dubble_jump.before(move_player))
        .add_system(change_player)
        .add_system(auto_step.before(move_player))
        .register_type::<Grounded>()
        .register_type::<Jump>()
        .register_type::<Player>();
    }
}

#[derive(Component, Reflect, PartialEq)]
pub enum Player {
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
    PhoxAnimationBundle::new(animation),
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
    Friction{
        coefficient: 5.,
        combine_rule: CoefficientCombineRule::Multiply,
    },
    Name::new("Player"),
    ));
}

const MOVE_SPEED: f32 = 100.;

fn move_player(
    mut player: Query<(&mut Velocity, &ActionState<PlayerInput>, &Grounded, &Transform), With<Player>>,
    rapier_context: Res<RapierContext>,
) {
    let (mut velocity, input, grounded, pos) = player.single_mut();
    if input.just_pressed(PlayerInput::Jump) & grounded {
        velocity.linvel.y = 100.;
    } else if input.just_pressed(PlayerInput::Fall) {
        velocity.linvel.y = velocity.linvel.y.min(0.0);
    } else if input.pressed(PlayerInput::Left) {
        let hit = rapier_context.cast_ray(pos.translation.truncate() + Vec2::new(-10., 16.), Vec2::NEG_Y, 31.9, false, QueryFilter::exclude_dynamic().exclude_sensors());
        if hit.is_none() {
            velocity.linvel.x = -MOVE_SPEED;
        }
    } else if input.pressed(PlayerInput::Right) {
        let hit = rapier_context.cast_ray(pos.translation.truncate() + Vec2::new(10., 16.), Vec2::NEG_Y, 31.9, false, QueryFilter::exclude_dynamic().exclude_sensors());
        if let Some(hit) = hit {
            info!("Player hit {:?}", hit.0);
            //velocity.linvel.x = 0.0
        }
        if hit.is_none() {
            velocity.linvel.x = MOVE_SPEED;
        }
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
pub struct Jump(pub bool);

#[derive(Component, Reflect)]
pub struct Grounded(bool);

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
    last.1 = last.1.clamp(0, 5);

    if last.1 == 5 && !on_ground.0 {
        on_ground.0 = true;
    } else if last.1 < 2 && on_ground.0 {
        on_ground.0 = false;
    }

    last.0 = (pos.translation.y * 100.).round();
}


impl std::ops::BitAnd<bool> for Grounded {
    type Output = bool;
    fn bitand(self, rhs: bool) -> Self::Output {
        self.0 & rhs
    }
}

impl std::ops::BitAnd<&Grounded> for bool {
    type Output = bool;
    fn bitand(self, rhs: &Grounded) -> Self::Output {
        self & rhs.0
    }
}

fn auto_step(
    mut query: Query<(&mut Transform, &ActionState<PlayerInput>, &Grounded), With<Player>>,
    rapier_context: Res<RapierContext>,
) {
    for (mut offset, state, grounded) in &mut query {
        if state.pressed(PlayerInput::Left) {
            let step = rapier_context.cast_ray(offset.translation.truncate() + Vec2::new(-10., 0.01), Vec2::NEG_Y, 15.9, true, QueryFilter::exclude_dynamic().exclude_sensors());
            if let Some((_, dis)) = step {
                let l_ray = rapier_context.cast_ray(offset.translation.truncate() + Vec2::new(-10., 7.99), Vec2::Y, 32., true, QueryFilter::exclude_dynamic().exclude_sensors());
                let r_ray = rapier_context.cast_ray(offset.translation.truncate() + Vec2::new(10., 7.99), Vec2::Y, 32., true, QueryFilter::exclude_dynamic().exclude_sensors());
                if let Some((bonk, at)) = l_ray {
                    println!("bonked ({:?} < {})", bonk, at)
                }
                if grounded.0 && l_ray.is_none() && r_ray.is_none() {
                    offset.translation.y += 16.1 - dis;
                }
            }
        } else if state.pressed(PlayerInput::Right) {
            let step = rapier_context.cast_ray(offset.translation.truncate() + Vec2::new(10., 0.01), Vec2::NEG_Y, 15.9, true, QueryFilter::exclude_dynamic().exclude_sensors());
            if let Some((_, dis)) = step {
                let r_ray = rapier_context.cast_ray(offset.translation.truncate() + Vec2::new(10., 7.99), Vec2::Y, 32., true, QueryFilter::exclude_dynamic().exclude_sensors());
                let l_ray = rapier_context.cast_ray(offset.translation.truncate() + Vec2::new(-10., 7.99), Vec2::Y, 32., true, QueryFilter::exclude_dynamic().exclude_sensors());
                if grounded.0 && l_ray.is_none() && r_ray.is_none() {
                    offset.translation.y += 16.1 - dis;
                }
            }
        }
    }
}