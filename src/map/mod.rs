use bevy::{prelude::*, reflect::TypePath};
use bevy_rapier2d::prelude::*;

mod collectable;
mod levels;
mod square;
mod tile_map;

mod prelude {
    pub use super::MapItem;
    use super::*;
    pub use collectable::{Collectable, CollectableType, SpawnType};
    pub use levels::Level;
    pub use square::Square;
    pub use tile_map::{MapData, MapEvent, MapObject, TerrainMaterial, TerrainType};
}

pub struct MapPlugin;

impl bevy::prelude::Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<MapEvent>()
            .add_systems(Update, tile_map::spawn_map_objects)
            .init_resource::<MapData>()
            .add_asset::<levels::Level>()
            .add_asset_loader(levels::LevelLoader)
            .init_resource::<LoadedLevel>()
            .add_systems(Update, load_map)
            .register_type::<Square>()
            .register_type::<TerrainMaterial>()
            .add_systems(Last, square::update_square);
    }
}

fn clear_old_map(mut map: ResMut<MapData>) {
    map.clear();
}

fn finish_map_update(mut map: ResMut<MapData>) {
    println!("Done: {}", map.need_correcting);
    map.need_correcting = false;
}

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum UpdateMap {
    ClearOld,
    Update,
    Finish,
}

#[derive(Bundle)]
struct CellBundle<T: MapObject + DrawProps + Component> {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub map_item: MapItem,
    pub item: T
}

impl<T: MapObject + Default + DrawProps + Component> Default for CellBundle<T> {
    fn default() -> Self {
        CellBundle {
            transform: default(),
            global_transform: default(),
            visibility: default(),
            computed_visibility: default(),
            collider: default(),
            rigid_body: default(),
            map_item: MapItem::new::<T>(),
            item: T::default()
        }
    }
}

pub use prelude::*;

use crate::{ghost::GhostEvents, player::RealPlayer};

use crate::editor::DrawProps;

#[derive(Resource, Default)]
pub struct LoadedLevel(pub Handle<Level>);

#[derive(Component, TypePath)]
pub struct MapItem(
    fn(root: Entity) -> belly::core::eml::Eml
);

impl MapItem {
    pub fn new<T: DrawProps>() -> MapItem {
        MapItem(T::draw_props)
    }
    pub fn draw_props(&self, root: Entity) -> belly::core::eml::Eml {
        (self.0)(root)
    }
}

fn load_map(
    mut map_event: EventWriter<MapEvent>,
    levels: Res<Assets<Level>>,
    current_level: Res<LoadedLevel>,
    map_item: Query<Entity, With<MapItem>>,
    mut commands: Commands,
    mut events: EventWriter<GhostEvents>,
    mut player: Query<&mut Transform, With<RealPlayer>>,
) {
    if !current_level.is_changed() {
        return;
    }
    let Some(level) = levels.get(&current_level.0) else {return;};
    events.send(GhostEvents::ClearGhosts);
    events.send(GhostEvents::ClearTrail);
    let mut player = player.single_mut();
    player.translation = level.player_start.as_vec2().extend(0.0);
    for item in &map_item {
        commands.entity(item).despawn_recursive();
    }
    for obj in level.objects.iter() {
        map_event.send(MapEvent::Spawn(MapObject::clone(obj.as_ref())))
    }
}
