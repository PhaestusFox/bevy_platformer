use crate::{
    animation::{Animation, Animations},
    map::{MapObject, MapItem},
    GameState, MainCam,
};
use belly::prelude::*;
pub use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LevelEditor), setup_editor);
        app.add_systems(OnExit(GameState::LevelEditor), cleanup_editor);
        app.add_systems(
            Update,
            (my_cursor_system).run_if(in_state(GameState::LevelEditor)),
        );
        app.add_systems(
            Update,
            setup_props.run_if(resource_exists_and_changed::<LastObj>()),
        );
    }
}

#[derive(Component)]
struct LevelRoot;

#[derive(Resource)]
pub struct LastObj(pub Option<Entity>);

fn setup_editor(mut commands: Commands) {
    println!("spawn");
    let mut childern = Vec::new();
    let editor = commands
        .spawn(NodeBundle::default())
        .with_children(|p| {
            childern.push(p.spawn(NodeBundle::default()).id());
        })
        .id();
    let next = childern.pop().expect("Exnugh childen for all objs");
    commands.add(crate::map::Square::ui_draw(next));
    commands.add(eml!(
        <body>
        <div c:level_editor {editor}>
        </div>
        <div c:object_editor>
            <label value="Test"/>
        </div>
        </body>
    ));
}

fn cleanup_editor(mut elements: Elements) {
    elements.select(".level_editor").remove();
    elements.select(".object_editor").remove();
}

fn my_cursor_system(
    // need to get window dimensions
    windows: Query<&Window>,
    primary: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    click: Res<Input<MouseButton>>,
) {
    if !click.just_pressed(MouseButton::Left) {
        return;
    }
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
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        println!(
            "World coords: {}/{}",
            (world_position.x / 16.).round(),
            (world_position.y / 16.).round()
        );
    }
}

fn setup_props(last: Res<LastObj>, mut elements: Elements, objs: Query<&MapItem>) {
    elements.select(".object_editor *").remove();
    if let Some(entity) = last.0 {
        // let obj = objs.get(entity).expect("Object to have item");
        // elements.select(".object_editor").add_child(obj.draw_props(entity));
    } else {
        elements.select(".object_editor").add_child(eml! {
            <label value="add/select Object to see its props"/>
        });
    }
}

pub trait DrawProps {
    fn draw_props(root: Entity) -> belly::core::eml::Eml;
    fn ui_draw(editor: Entity) -> belly::core::eml::Eml;
}

// fn spawn_new_obj(
//     mut commands: Commands,
//     editor_state: Res<LevelEditorState>,
//     children: Query<&Children>,
// ) {
//     if !editor_state.is_changed() { return; }
//     if let Some(obj) = &editor_state.current {
//         if let Ok(c) = children.get(editor_state.root) {
//             for child in c {
//                 commands.entity(*child).despawn_recursive();
//             }
//         }
//         obj.ui_draw(commands.entity(editor_state.root));
//     }
// }
