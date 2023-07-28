use crate::{
    map::{Level, LoadedLevel},
    GameState,
};
use belly::{core::input::Focused, prelude::*};
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuFont>()
            .add_systems(Startup, setup_belly)
            .add_systems(
                StateTransition,
                cleanup_old
                    .run_if(state_will_change::<GameState>)
                    .before(apply_state_transition::<GameState>),
            )
            .add_systems(OnEnter(GameState::Menu), setup_main_menu)
            .add_systems(OnEnter(GameState::InputLevelBase64), setup_level_select)
            .add_systems(OnEnter(GameState::InputLevelName), setup_level_select)
            .add_systems(
                OnTransition {
                    from: GameState::InputLevelBase64,
                    to: GameState::Play,
                },
                load_base64_level,
            )
            .add_systems(
                OnTransition {
                    from: GameState::InputLevelName,
                    to: GameState::Play,
                },
                load_name_level,
            );
    }
}

fn state_will_change<T: States>(state: Res<NextState<T>>) -> bool {
    state.0.is_some()
}

fn setup_belly(mut commands: Commands) {
    commands.add(StyleSheet::load("main.ess"));
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

fn cleanup_old(mut elements: Elements, mut focus: ResMut<Focused>) {
    focus.0 = None;
    elements.select(".menu").remove()
}

fn setup_main_menu(mut commands: Commands) {
    commands.add(eml! {
        <div c:menu>
            <button on:press=run!(|ctx| {
                ctx.commands().add(move |world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::Play);
                    println!("{:?}", world.resource::<State<GameState>>());
                });
            }) value="play"><label value="Play"/></button>
            <button on:press=run!(|ctx| {
                ctx.commands().add(move |world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::InputLevelBase64);
                });
            }) value="base"><label value="Base64"/></button>
            <button on:press=run!(|ctx| {
                ctx.commands().add(move |world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::InputLevelName);
                });
            }) value="name"><label value="Name"/></button>
            <button on:press=run!(|ctx| {
                ctx.commands().add(move |world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::LevelEditor);
                });
            }) value="editor"><label value="Level Editor"/></button>
        </div>
    });
}

fn setup_level_select(mut commands: Commands) {
    commands.add(eml! {
        <div c:menu>
            <textinput />
            <button value="play" on:press=run!(|ctx| {
                ctx.commands().add(move |world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::Play);
                    println!("{:?}", world.resource::<State<GameState>>());
                });
            })><label value="Play"/></button>
        </div>
    });
}

fn load_base64_level(
    mut levels: ResMut<Assets<Level>>,
    mut elements: Elements,
    mut loaded_level: ResMut<LoadedLevel>,
    query: Query<&TextInput>,
) {
    let data = *elements
        .select(".menu textinput")
        .entities()
        .first()
        .expect("textinput in menu");
    let textinput = query.get(data).expect("textinput is not TextInput");
    match Level::from_base64(&textinput.value) {
        Ok(level) => {
            loaded_level.0 = levels.add(level);
        }
        Err(e) => {
            error!("{}", e);
        }
    };
}

fn load_name_level(
    asset_server: Res<AssetServer>,
    mut elements: Elements,
    mut loaded_level: ResMut<LoadedLevel>,
    query: Query<&TextInput>,
) {
    let data = *elements
        .select(".menu textinput")
        .entities()
        .first()
        .expect("textinput in menu");
    let textinput = query.get(data).expect("textinput is not TextInput");
    loaded_level.0 = asset_server.load(&textinput.value);
}
