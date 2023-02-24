use bevy::{prelude::*};

use crate::{GameState, map::{Level, LoadedLevel}};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuFont>()
        .add_system_set(SystemSet::on_enter(GameState::Menu)
        .with_system(setup_main_menu))
        .add_system_set(SystemSet::on_update(GameState::Menu)
        .with_system(state_buttons))
        .add_system_set(SystemSet::on_exit(GameState::Menu)
        .with_system(cleanup_menu))
        .add_system_set(SystemSet::on_enter(GameState::InputLevelBase64)
        .with_system(setup_level_select)
        .with_system(|mut editor: ResMut<bevy_editor_pls::EditorState>| editor.listening_for_text = true))
        .add_system_set(SystemSet::on_update(GameState::InputLevelBase64)
        .with_system(load_base64_level)
        .with_system(input_button))
        .add_system_set(SystemSet::on_exit(GameState::InputLevelBase64)
        .with_system(|mut editor: ResMut<bevy_editor_pls::EditorState>| editor.listening_for_text = false)
        .with_system(cleanup_menu))

        .add_system_set(SystemSet::on_enter(GameState::InputLevelName)
        .with_system(setup_level_select)
        .with_system(|mut editor: ResMut<bevy_editor_pls::EditorState>| editor.listening_for_text = true))
        .add_system_set(SystemSet::on_update(GameState::InputLevelName)
        .with_system(load_name_level)
        .with_system(input_button))
        .add_system_set(SystemSet::on_exit(GameState::InputLevelName)
        .with_system(|mut editor: ResMut<bevy_editor_pls::EditorState>| editor.listening_for_text = false)
        .with_system(cleanup_menu))

        .insert_resource(LevelString(String::new()));
    }
}

#[derive(Component)]
struct MenuItem;

#[derive(Debug, Resource)]
struct MenuFont(Handle<Font>);
impl FromWorld for MenuFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        MenuFont(asset_server.load("Raleway-Regular.ttf"))
    }
}

fn make_button<T: Bundle>(parent: &mut ChildBuilder, style: Style, text: &str, font: Handle<Font>, components: T) {
    parent.spawn((ButtonBundle{
        style,
        background_color: Color::DARK_GRAY.into(),
        ..Default::default()
    }, components)).with_children(|p| {
    p.spawn(TextBundle {
            text: Text { sections: vec![TextSection {
                value: text.to_string(),
                style: TextStyle { font, font_size: 20., color: Color::WHITE }
            }], alignment: TextAlignment::CENTER },
            ..Default::default()
        });
    });
}

fn setup_main_menu(
    mut commands: Commands,
    font: Res<MenuFont>,
) {
    commands.spawn((NodeBundle {
        style: Style {
            margin: UiRect::all(Val::Auto),
            size: Size::new(Val::Percent(25.), Val::Percent(66.)),
            flex_wrap: FlexWrap::Wrap,
            ..Default::default()
        },
        background_color: Color::GRAY.into(),
        ..Default::default()
    }, MenuItem))
    .with_children(|p| {
        let size = Style {
            padding: UiRect::top(Val::Px(10.)),
            size: Size::new(Val::Percent(100.),Val::Percent(20.)),
            ..Default::default()
        };
        Size::new(Val::Percent(100.), Val::Percent(20.));
        make_button(p, size.clone(),"Play", font.0.clone(), GameState::Play);
        make_button(p, size.clone(),"Base64", font.0.clone(), GameState::InputLevelBase64);
        make_button(p, size,"CustomName", font.0.clone(), GameState::InputLevelName);
    });
}

fn setup_level_select(
    mut commands: Commands,
    font: Res<MenuFont>,
    mut level_string: ResMut<LevelString>,
    mut loaded_level: ResMut<LoadedLevel>,
) {
    level_string.0.clear();
    loaded_level.0 = Handle::default();
    commands.spawn((NodeBundle {
        style: Style {
            margin: UiRect::all(Val::Auto),
            size: Size::new(Val::Percent(66.), Val::Percent(50.)),
            flex_wrap: FlexWrap::Wrap,
            ..Default::default()
        },
        background_color: Color::GRAY.into(),
        ..Default::default()
    }, MenuItem))
    .with_children(|p| {
        let size = Style {
            padding: UiRect::top(Val::Px(10.)),
            size: Size::new(Val::Percent(100.),Val::Percent(20.)),
            ..Default::default()
        };
        Size::new(Val::Percent(100.), Val::Percent(20.));
        make_button(p, size.clone(),"", font.0.clone(), GameState::InputLevelBase64);
        make_button(p, size.clone(),"Play", font.0.clone(), GameState::Play);
        make_button(p, size,"", font.0.clone(), InputError);
    });
}

fn cleanup_menu(
    mut commands: Commands,
    query: Query<Entity, With<MenuItem>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn state_buttons(
    query: Query<(&Interaction, &GameState), (With<Button>, Changed<Interaction>)>,
    mut state: ResMut<State<GameState>>,
) {
    for (interaction, next_state) in &query {
        match interaction {
            Interaction::Clicked => {let _ = state.set(*next_state);},
            _ => {},
        }
    }
}

#[derive(Component)]
struct InputError;

#[derive(Resource)]
struct LevelString(String);

fn load_base64_level(
    mut levels: ResMut<Assets<Level>>,
    level: Res<LevelString>,
    mut loaded_level: ResMut<LoadedLevel>,
    query: Query<(&Interaction, &GameState), (With<Button>, Changed<Interaction>)>,
    mut text: Query<&mut Text>,
    error: Query<&Children, With<InputError>>,
    mut state: ResMut<State<GameState>>,
) {
    for (interaction, gamestate) in &query {
        if GameState::Play != *gamestate {continue;}
        match interaction {
            Interaction::Clicked => {
                match Level::from_base64(&level.0) {
                    Ok(level) => {
                        *loaded_level = LoadedLevel(levels.add(level));
                        let _ = state.set(GameState::Play);
                    },
                    Err(e) => {
                        for children in error.iter() {
                            for child in children {
                                if let Ok(mut text) = text.get_mut(*child) {
                                    text.sections[0].value = e.to_string();
                                    text.sections[0].style.color = Color::RED;
                                }
                            }
                        }
                        error!("{:?}", e.to_string());
                        return;
                    }
                }
            },
            _ => {}
        }
    }
}

fn input_button(
    mut text: Query<&mut Text>,
    mut inputs: Query<(&Children, &mut Style, &GameState)>,
    mut keys: EventReader<ReceivedCharacter>,
    mut buttons: Query<(&mut Interaction, &GameState)>,
    mut output: ResMut<LevelString>,
) {
    for key in keys.iter() {
        if key.char as u8 == 22 {
            if let Ok(str) = cli_clipboard::get_contents() {
                for c in str.chars() {
                    if c.is_alphanumeric() || c == '+' || c == '/' || c == '=' {
                        if output.0.len() != 0 && output.0.len() % 89 == 0 {
                            output.0.push('\n');
                        }
                        output.0.push(c);
                    }
                }
            }
        }
        if key.char as u8 == 13 {
            for (mut button, state) in &mut buttons {
                if *state == GameState::Play {*button = Interaction::Clicked;}
                return;
            }
        }
        if key.char.is_alphanumeric() || key.char == '+' || key.char == '/' || key.char == '=' {
            if output.0.len() != 0 && output.0.len() % 89 == 0 {
                output.0.push('\n');
            }
            output.0.push(key.char);
        }
        if key.char == '\u{8}' {
            output.0.pop();
            if let Some('\n') = output.0.chars().last() {output.0.pop();}
        }
    }
    if output.is_changed() {
        for (children,mut style, gamestate) in &mut inputs {
            if *gamestate != GameState::InputLevelBase64 {continue;}
            style.size.height = Val::Percent(20. + (output.0.len()/90) as f32 * 10.);
            for child in children.iter() {
                if let Ok(mut text) = text.get_mut(*child) {
                    text.sections[0].value = output.0.clone();
                }
            }
        }
    }
}

fn load_name_level(
    asset_server: Res<AssetServer>,
    level: Res<LevelString>,
    mut loaded_level: ResMut<LoadedLevel>,
    query: Query<(&Interaction, &GameState), (With<Button>, Changed<Interaction>)>,
    mut text: Query<&mut Text>,
    error: Query<&Children, With<InputError>>,
    mut state: ResMut<State<GameState>>,
) {
    for (interaction, gamestate) in &query {
        if GameState::Play != *gamestate {continue;}
        match interaction {
            Interaction::Clicked => {
                loaded_level.0 = asset_server.load(format!("levels/{}.lvl.ron", level.0));
                for children in error.iter() {
                    for child in children {
                        if let Ok(mut text) = text.get_mut(*child) {
                            text.sections[0].value = "".to_string();
                            text.sections[0].style.color = Color::RED;
                        }
                    }
                }
            },
            _ => {}
        }
    }
    match asset_server.get_load_state(&loaded_level.0) {
        bevy::asset::LoadState::Loaded => {
            let _ = state.set(GameState::Play);
        },
        bevy::asset::LoadState::Failed => {
            for children in error.iter() {
                for child in children {
                    if let Ok(mut text) = text.get_mut(*child) {
                        text.sections[0].value = "Faild to load level".to_string();
                        text.sections[0].style.color = Color::RED;
                    }
                }
            }
        },
        _ => {}
    }
}
