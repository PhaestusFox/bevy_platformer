use bevy::prelude::*;
use std::collections::HashMap;

use bevy_rapier2d::prelude::*;
use crate::{player::{Player, Grounded, Jump, RealPlayer, GroundedCheck, PlayerStages}, user_input::PlayerInput, animation::{Animations, PhoxAnimationBundle, Animation}};
use leafwing_input_manager::prelude::*;

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerInputs>()
            .init_resource::<SyncOffset>()
            .insert_resource(PlayerFrame(0))
            .add_system_to_stage(CoreStage::First, update_frame)
            .add_system_to_stage(CoreStage::Last, save_player_inputs)
            .add_system_to_stage(CoreStage::Last, save_player_offset)
            .add_system(update_ghost.before(PlayerStages::Move))
            .add_system_to_stage(CoreStage::Last, drift_correct)
            .add_system(test_ghost);
    }
}

#[derive(Component)]
pub struct Ghost(usize);

const SYNCFRAME: usize = 3;

#[derive(Resource)]
struct PlayerFrame(usize);

#[derive(Resource, Default)]
struct PlayerInputs(HashMap<usize, ActionState<PlayerInput>>);

impl PlayerInputs {
    fn add_input(&mut self, frame: usize, state: ActionState<PlayerInput>) {
        self.0.insert(frame, state);
    }
    fn get_input(&self, frame: &usize) -> Option<&ActionState<PlayerInput>> {
        self.0.get(frame)
    }
}

#[derive(Resource, Default)]
struct SyncOffset(HashMap<usize, Vec3>);

impl SyncOffset {
    fn add_offset(&mut self, frame: usize, state: Vec3) {
        self.0.insert(frame, state);
    }
    fn get_offset(&self, frame: &usize) -> Option<&Vec3> {
        self.0.get(&frame)
    }
}

fn update_frame(
    mut frame: ResMut<PlayerFrame>,
    mut query: Query<&mut Ghost>,
) {
    for mut frame in query.iter_mut() {
        frame.0 += 1;
    }
    frame.0 += 1;
}

fn save_player_inputs(
    query: Query<&ActionState<PlayerInput>, With<RealPlayer>>,
    frame: Res<PlayerFrame>,
    mut inputs: ResMut<PlayerInputs>,
) {
    let player = query.single();
    inputs.add_input(frame.0, player.clone());
}

fn save_player_offset(
    query: Query<&Transform, With<RealPlayer>>,
    frame: Res<PlayerFrame>,
    mut offsets: ResMut<SyncOffset>,
) {
    if frame.0 % SYNCFRAME == 0 {
        let player = query.single();
        offsets.add_offset(frame.0, player.translation);
    }
}

fn update_ghost(
    mut ghosts: Query<(&mut ActionState<PlayerInput>, &Ghost)>,
    inputs: Res<PlayerInputs>,
) {
    for (mut map, Ghost(frame) ) in &mut ghosts {
        if let Some(new_map) = inputs.get_input(frame) {
            if new_map.just_pressed(PlayerInput::Jump) {
                println!("Ghost jump");
            }
            *map = new_map.clone();
        }
    }
}

fn test_ghost(
    input: Res<Input<KeyCode>>,
    mut commands: Commands,
    animations: Res<Animations>,
) {
    if input.just_pressed(KeyCode::Escape) {
        let Some((texture_atlas, animation)) = animations.get(Animation::MaskIdle) else {error!("Failed to find animation: Idle"); return;};
        commands.spawn((
            (SpriteSheetBundle {
                texture_atlas,
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..Default::default()
                },
                ..Default::default()
            },
            Player::Mask,
            PhoxAnimationBundle::new(animation),
            Grounded(true),
            GroundedCheck::default(),
            ActionState::<PlayerInput>::default(),
            Jump(false),
            RigidBody::Dynamic,
            Velocity::default(),
            Collider::cuboid(9., 16.),
            LockedAxes::ROTATION_LOCKED_Z,
            Friction {
                coefficient: 5.,
                combine_rule: CoefficientCombineRule::Multiply,
            },
            Damping {
                linear_damping: 1.,
                angular_damping: 1.,
            },
            Name::new("Ghost"),
            Ghost(0)),
            CollisionGroups::new(Group::GROUP_2, Group::GROUP_1),
        ));
    }
}

fn drift_correct(
    mut query: Query<(&Ghost, &mut Transform)>,
    offsets: Res<SyncOffset>,
) {
    for (Ghost(frame), mut transform) in &mut query {
        if frame % SYNCFRAME != 0 {continue;}
        let Some(offset) = offsets.get_offset(frame) else {error!("No Sync for frame {}", frame); continue;};
        transform.translation = *offset;
    }
}