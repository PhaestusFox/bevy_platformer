use bevy::{ecs::query::QuerySingleError, prelude::*};

use crate::{
    animation::{Animation, Animations},
    map::LoadedLevel,
    player::{Grounded, GroundedCheck, Jump, Player, PlayerStages, RealPlayer},
    user_input::PlayerInput,
    Score,
};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputs>()
            .init_resource::<SyncOffset>()
            .insert_resource(PlayerFrame(0))
            .add_systems(First, update_frame)
            .add_systems(Last, save_player_state)
            .add_systems(Last, save_player_offset)
            .add_systems(Last, drift_correct)
            .add_systems(Update, update_ghost.before(PlayerStages::Move))
            .add_systems(Update, test_ghost)
            .add_event::<GhostEvents>()
            .add_systems(Update, handle_ghost_event)
            .add_systems(Update, kill_player)
            .add_systems(Update, auto_ghost);
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

fn update_frame(mut frame: ResMut<PlayerFrame>, mut query: Query<&mut Ghost>) {
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
    for (mut v, mut j, mut p, &Ghost(frame)) in &mut ghosts {
        if frame % 600 == 0 {
            println!(
                "PlayerInputs = {}",
                inputs.0.len() * size_of::<(Velocity, Jump)>()
            );
        }
        if let Some((new_v, new_j, new_p)) = inputs.get_input(frame) {
            *v = new_v.clone();
            *j = *new_j;
            *p = *new_p;
        }
    }
}

fn test_ghost(input: Res<Input<KeyCode>>, mut events: EventWriter<GhostEvents>) {
    if input.just_pressed(KeyCode::Escape) {
        events.send(GhostEvents::SpawnGhost);
    }
    if input.just_pressed(KeyCode::F5) {
        events.send(GhostEvents::ClearGhosts);
        events.send(GhostEvents::ClearTrail);
    }
}

fn drift_correct(mut query: Query<(&Ghost, &mut Transform)>, offsets: Res<SyncOffset>) {
    use std::mem::size_of;
    for (&Ghost(frame), mut transform) in &mut query {
        if frame % 600 == 0 {
            println!("offsets = {}", offsets.0.len() * size_of::<Vec3>());
        }
        if frame % SYNCFRAME != 0 || frame == 0 {
            continue;
        }
        let Some(offset) = offsets.get_offset((frame - 1) / SYNCFRAME) else {error!("No Sync for frame {}", frame); continue;};
        transform.translation = *offset;
    }
}

fn handle_ghost_event(
    mut events: EventReader<GhostEvents>,
    mut frame: ResMut<PlayerFrame>,
    mut inputs: ResMut<PlayerInputs>,
    mut offsets: ResMut<SyncOffset>,
    mut commands: Commands,
    ghosts: Query<Entity, With<Ghost>>,
    animations: Res<Animations>,
) {
    for event in events.iter() {
        match event {
            GhostEvents::ClearTrail => {
                frame.0 = 1;
                inputs.0.clear();
                offsets.0.clear();
            }
            GhostEvents::ClearGhosts => {
                for ghost in &ghosts {
                    commands.entity(ghost).despawn();
                }
            }
            GhostEvents::SpawnGhost => {
                let Some(handle) = animations.get_animation(Animation::MaskIdle) else {error!("Failed to find animation: Idle"); return;};
                commands.spawn((
                    (
                        SpriteSheetBundle {
                            texture_atlas: Handle::default(),
                            sprite: TextureAtlasSprite {
                                index: 0,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        Player::Mask,
                        handle,
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
                        Ghost(0),
                    ),
                    CollisionGroups::new(Group::GROUP_2, Group::GROUP_1),
                ));
            }
        }
    }
}

#[derive(Event)]
pub enum GhostEvents {
    ClearTrail,
    ClearGhosts,
    SpawnGhost,
}

fn kill_player(
    rapier_context: Res<RapierContext>,
    mut player: Query<(Entity, &mut Transform, &mut Velocity), With<RealPlayer>>,
    ghosts: Query<Entity, With<Ghost>>,
    mut events: EventWriter<GhostEvents>,
    mut score: ResMut<Score>,
    mut loaded_level: ResMut<LoadedLevel>,
) {
    let (player, mut pos, mut vel) = player.single_mut();
    for ghost in &ghosts {
        let Some(contact) = rapier_context.contact_pair(player, ghost) else { continue;};
        if contact.has_any_active_contacts() {
            println!("score = {}", score.0);
            score.0 = 0;
            events.send(GhostEvents::ClearGhosts);
            events.send(GhostEvents::ClearTrail);
            *vel = Velocity::zero();
            *pos = Transform::IDENTITY;
            loaded_level.set_changed();
        };
    }
}

struct GhostTimer(Timer);
impl Default for GhostTimer {
    fn default() -> Self {
        GhostTimer(Timer::from_seconds(2.5, TimerMode::Once))
    }
}

fn auto_ghost(
    has_ghost: Query<&Ghost>,
    player: Query<&Transform, With<RealPlayer>>,
    mut count_down: Local<GhostTimer>,
    mut events: ParamSet<(EventReader<GhostEvents>, EventWriter<GhostEvents>)>,
    time: Res<Time>,
) {
    for event in events.p0().iter() {
        match event {
            GhostEvents::ClearGhosts => {
                count_down.0.reset();
            }
            _ => {}
        }
    }
    if let Err(QuerySingleError::NoEntities(_)) = has_ghost.get_single() {
        let player = player.single();
        if player.translation.distance(Vec3::ZERO) < 8. && !count_down.0.finished() {
            events.p1().send(GhostEvents::ClearTrail);
            count_down.0.reset();
            return;
        }
        count_down.0.tick(time.delta());
        if count_down.0.finished() {
            events.p1().send(GhostEvents::SpawnGhost);
        }
    }
}
