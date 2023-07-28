use crate::animation::{Animation, Animations};
use crate::user_input::PlayerInput;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum PlayerStages {
    Move,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, move_player.in_set(PlayerStages::Move))
            .add_systems(Update, ground_detection)
            .add_systems(
                Update,
                dubble_jump.in_set(PlayerStages::Move).before(move_player),
            )
            .add_systems(Update, change_player.in_set(PlayerStages::Move))
            .add_systems(
                Update,
                auto_step.in_set(PlayerStages::Move).before(move_player),
            )
            .register_type::<Grounded>()
            .register_type::<Jump>()
            .register_type::<Player>();
    }
}

#[derive(Component, Reflect, PartialEq, Clone, Copy)]
pub enum Player {
    Mask,
    Ninja,
    Pink,
    Guy,
}

#[derive(Component)]
pub struct RealPlayer;

fn spawn_player(mut commands: Commands, animations: Res<Animations>) {
    let Some(handle) = animations.get_animation(Animation::MaskIdle) else {error!("Failed to find animation: Idle"); return;};
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: default(),
            sprite: TextureAtlasSprite {
                index: 0,
                ..Default::default()
            },
            ..Default::default()
        },
        Player::Mask,
        RealPlayer,
        handle,
        Grounded(true),
        GroundedCheck(0.0, 0),
        InputManagerBundle {
            input_map: PlayerInput::player_one(),
            ..Default::default()
        },
        Jump(false),
        RigidBody::Dynamic,
        Velocity::default(),
        Collider::cuboid(9., 15.95),
        LockedAxes::ROTATION_LOCKED_Z,
        Friction {
            coefficient: 5.,
            combine_rule: CoefficientCombineRule::Multiply,
        },
        Damping {
            linear_damping: 1.,
            angular_damping: 1.,
        },
        Name::new("Player"),
    ));
}

pub const MAX_SPEED: f32 = 200.;
pub const ACCELERATION: f32 = 50.;

fn move_player(
    mut player: Query<(
        &mut Velocity,
        &ActionState<PlayerInput>,
        &Grounded,
        &Transform,
    )>,
    rapier_context: Res<RapierContext>,
) {
    for (mut velocity, input, grounded, pos) in &mut player {
        if input.just_pressed(PlayerInput::Jump) & grounded {
            velocity.linvel.y = 250.;
        } else if input.just_pressed(PlayerInput::Fall) {
            velocity.linvel.y = velocity.linvel.y.min(0.0);
        } else if input.pressed(PlayerInput::Left) {
            let hit = rapier_context.cast_ray(
                pos.translation.truncate() + Vec2::new(-10., 16.),
                Vec2::NEG_Y,
                31.,
                false,
                QueryFilter::exclude_dynamic().exclude_sensors(),
            );
            if hit.is_none() {
                velocity.linvel.x -= ACCELERATION;
            }
        } else if input.pressed(PlayerInput::Right) {
            let hit = rapier_context.cast_ray(
                pos.translation.truncate() + Vec2::new(10., 16.),
                Vec2::NEG_Y,
                31.,
                false,
                QueryFilter::exclude_dynamic().exclude_sensors(),
            );
            if hit.is_none() {
                velocity.linvel.x += ACCELERATION;
            }
        };
        velocity.linvel.x = velocity.linvel.x.clamp(-MAX_SPEED, MAX_SPEED);
    }
}

fn dubble_jump(
    mut player: Query<(&mut Jump, &mut Velocity, &ActionState<PlayerInput>)>,
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
        if velocity.linvel.y.abs() < 0.01 {
            return;
        }
        if input.just_pressed(PlayerInput::Jump) && jump.0 {
            jump.0 = false;
            velocity.linvel.y = 250.;
        }
    }
}

fn change_player(mut query: Query<(&mut Player, &ActionState<PlayerInput>)>) {
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

#[derive(Component, Reflect, Clone, Copy)]
pub struct Jump(pub bool);

#[derive(Component, Reflect)]
pub struct Grounded(pub bool);
#[derive(Component, Default)]
pub struct GroundedCheck(f32, isize);

fn ground_detection(mut player: Query<(&Transform, &mut Grounded, &mut GroundedCheck)>) {
    for (pos, mut on_ground, mut last) in &mut player {
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
    mut query: Query<(&mut Transform, &ActionState<PlayerInput>, &Grounded)>,
    rapier_context: Res<RapierContext>,
) {
    for (mut offset, state, grounded) in &mut query {
        if state.pressed(PlayerInput::Left) {
            let step = rapier_context.cast_ray(
                offset.translation.truncate() + Vec2::new(-10., 0.01),
                Vec2::NEG_Y,
                15.9,
                true,
                QueryFilter::exclude_dynamic().exclude_sensors(),
            );
            if let Some((_, dis)) = step {
                let l_ray = rapier_context.cast_ray(
                    offset.translation.truncate() + Vec2::new(-10., 7.99),
                    Vec2::Y,
                    32.,
                    true,
                    QueryFilter::exclude_dynamic().exclude_sensors(),
                );
                let r_ray = rapier_context.cast_ray(
                    offset.translation.truncate() + Vec2::new(10., 7.99),
                    Vec2::Y,
                    32.,
                    true,
                    QueryFilter::exclude_dynamic().exclude_sensors(),
                );
                if grounded.0 && l_ray.is_none() && r_ray.is_none() {
                    offset.translation.y += 16.1 - dis;
                }
            }
        } else if state.pressed(PlayerInput::Right) {
            let step = rapier_context.cast_ray(
                offset.translation.truncate() + Vec2::new(10., 0.01),
                Vec2::NEG_Y,
                15.9,
                true,
                QueryFilter::exclude_dynamic().exclude_sensors(),
            );
            if let Some((_, dis)) = step {
                let r_ray = rapier_context.cast_ray(
                    offset.translation.truncate() + Vec2::new(10., 7.99),
                    Vec2::Y,
                    32.,
                    true,
                    QueryFilter::exclude_dynamic().exclude_sensors(),
                );
                let l_ray = rapier_context.cast_ray(
                    offset.translation.truncate() + Vec2::new(-10., 7.99),
                    Vec2::Y,
                    32.,
                    true,
                    QueryFilter::exclude_dynamic().exclude_sensors(),
                );
                if grounded.0 && l_ray.is_none() && r_ray.is_none() {
                    offset.translation.y += 16.1 - dis;
                }
            }
        }
    }
}
