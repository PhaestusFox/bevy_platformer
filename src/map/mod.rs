use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod tile_map;
mod collectable;
mod square;

mod prelude {
    use super::*;
    pub use tile_map::{MapData, MapObject, MapEvent, TerrainMaterial, TerrainType};
    pub use square::MapBox;
    pub use collectable::{Collectable, CollectableType, SpawnType};
}

pub struct MapPlugin;

impl bevy::prelude::Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<MapEvent>()
            .add_system(tile_map::spawn_map_objects)
            .init_resource::<MapData>();
    }
}

#[derive(Bundle, Default)]
struct CellBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub collider: Collider,
    pub rigid_body: RigidBody,
}


pub use prelude::*;