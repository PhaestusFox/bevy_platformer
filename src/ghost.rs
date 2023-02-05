use bevy::prelude::*;

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
            .add_system_to_stage(CoreStage::Last, save_player_state)
            .add_system_to_stage(CoreStage::Last, save_player_offset)
            .add_system(update_ghost.before(PlayerStages::Move))
            .add_system_to_stage(CoreStage::Last, drift_correct)
            .add_system(test_ghost);
    }
}

#[derive(Component)]
pub struct Ghost(usize);

const SYNCFRAME: usize = 10;

#[derive(Resource)]
struct PlayerFrame(usize);

#[derive(Resource, Default)]
struct PlayerInputs(Vec<(Velocity, Jump, Player)>);

impl PlayerInputs {
    fn add_input(&mut self, state: (Velocity, Jump, Player)) {
        self.0.push(state);
    }
    fn get_input(&self, frame: usize) -> Option<&(Velocity, Jump, Player)> {
        self.0.get(frame)
    }
}

#[derive(Resource, Default)]
struct SyncOffset(Vec<Vec3>);

impl SyncOffset {
    fn add_offset(&mut self, state: Vec3) {
        self.0.push(state);
    }
    fn get_offset(&self, frame: usize) -> Option<&Vec3> {
        self.0.get(frame)
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

fn save_player_state(
    query: Query<(&Velocity, &Jump, &Player), With<RealPlayer>>,
    mut inputs: ResMut<PlayerInputs>,
) {
    let player = query.single();
    inputs.add_input((player.0.clone(), *player.1, *player.2));
}

fn save_player_offset(
    query: Query<&Transform, With<RealPlayer>>,
    frame: Res<PlayerFrame>,
    mut offsets: ResMut<SyncOffset>,
) {
    if frame.0 % SYNCFRAME == 0 {
        let player = query.single();
        offsets.add_offset(player.translation);
    }
}

fn update_ghost(
    mut ghosts: Query<(&mut Velocity, &mut Jump, &mut Player, &Ghost)>,
    inputs: Res<PlayerInputs>,
) {
    use std::mem::size_of;
    for (mut v, mut j, mut p, &Ghost(frame) ) in &mut ghosts {
        if frame % 600 == 0 {
            println!("PlayerInputs = {}", inputs.0.len() * size_of::<(Velocity, Jump)>());
        }
        if let Some((new_v, new_j, new_p)) = inputs.get_input(frame) {
            *v = new_v.clone();
            *j = *new_j;
            *p = *new_p;
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
    use std::mem::size_of;
    for (&Ghost(frame), mut transform) in &mut query {
        if frame % 600 == 0 {
            println!("offsets = {}", offsets.0.len() * size_of::<Vec3>());
        }
        if frame % SYNCFRAME != 0 || frame == 0 {continue;}
        let Some(offset) = offsets.get_offset((frame - 1) / SYNCFRAME) else {error!("No Sync for frame {}", frame); continue;};
        transform.translation = *offset;
    }
}