pub use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

use crate::{map::MapObject, GameState, animation::{Animations, Animation}, MainCam};

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LevelEditor), setup_level_editor);
        app.add_systems(OnExit(GameState::LevelEditor), cleanup_editor);
        app.add_systems(Update, (my_cursor_system, spawn_new_obj).run_if(in_state(GameState::LevelEditor)));
    }
}

fn make_item_button<C: Bundle, M: Bundle>(
    id: IVec2,
    commands: &mut Commands,
    main: M,
    child: C,
    parent: Entity,
) {
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(IVec2::new(-75 + id.x * 100, 310 - id.y * 100).as_vec2().extend(1.)),
            ..Default::default()
        },
        Collider::cuboid(50., 50.),
        main
    ))
    .with_children(|p| {
        p.spawn(child);
    })
    .set_parent(parent);
}

#[derive(Resource)]
pub struct LevelEditorState {
    current: Option<Box<dyn MapObject>>,
    root: Entity,
}

fn setup_level_editor(
    mut commands: Commands,
    animations: Res<Animations>,
    asset_server: Res<AssetServer>,
) {
    let level_editor = LevelEditorState {
        root: commands.spawn(NodeBundle::default()).id(),
        current: None,
    };
    let p = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::GRAY,
                custom_size: Some(Vec2 { x: 250., y: 720. }),
                ..Default::default()
            },
            texture: asset_server.load("ui_buttion.png"),
            transform: Transform::from_translation(Vec3::new(((1280/2) - 125) as f32, 0., 0.)),
            ..Default::default()
        },
        MainEditor,
        Name::new("EditorWindow")
    )).id();
    make_item_button(IVec2::new(0,0), &mut commands, (Sprite {
        color: Color::DARK_GRAY,
        custom_size: Some(Vec2 { x: 100., y: 100. }),
        ..Default::default()
    },
    asset_server.load::<Image, _>("ui_buttion.png"),), (Handle::<TextureAtlas>::default(), TextureAtlasSprite{custom_size: Some(Vec2::splat(50.)), ..Default::default() }, animations.get_animation(Animation::Strawberry).expect("Animation loaded"), SpatialBundle::default()), p);
    commands.insert_resource(level_editor);
}

fn cleanup_editor(
    mut commands: Commands,
    level_editor: Res<LevelEditorState>,
    query: Query<Entity, Or<(With<MainEditor>, With<SubEditor>, With<Shadow>)>>,
) {
    commands.remove_resource::<LevelEditorState>();
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.entity(level_editor.root).despawn_recursive();
}

#[derive(Component)]
struct MainEditor;

#[derive(Component)]
struct SubEditor;

#[derive(Component)]
struct Shadow;

fn my_cursor_system(
    // need to get window dimensions
    windows: Query<&Window>,
    primary: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    click: Res<Input<MouseButton>>,
) {
    if !click.just_pressed(MouseButton::Left) {return;}
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = camera_q.single();

    // get the window that the camera is displaying to (or the primary window)
    let window = if let bevy::render::camera::RenderTarget::Window(id) = camera.target {
        match id {
            bevy::window::WindowRef::Primary => primary.single(),
            bevy::window::WindowRef::Entity(e) => windows.get(e).unwrap(),
        }
    } else {
        primary.single()
    };

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        println!("World coords: {}/{}", (world_position.x / 16.).round(), (world_position.y / 16.).round());
    }
}

fn spawn_new_obj(
    mut commands: Commands,
    editor_state: Res<LevelEditorState>,
    children: Query<&Children>,
) {
    if !editor_state.is_changed() { return; }
    if let Some(obj) = &editor_state.current {
        if let Ok(c) = children.get(editor_state.root) {
            for child in c {
                commands.entity(*child).despawn_recursive();
            }
        }
        obj.ui_draw(commands.entity(editor_state.root));
    }
}