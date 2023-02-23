use std::collections::HashSet;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::animation::Animations;

pub enum MapEvent {
    Spawn(Box<dyn MapObject>),
}

impl MapEvent {
    pub fn spawn(object: impl MapObject) -> MapEvent {
        MapEvent::Spawn(Box::new(object))
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, Reflect)]
#[reflect_value(Serialize)]
#[allow(dead_code)]
pub enum TerrainMaterial {
    Gold = 193,
    Brick = 105,
    Copper = 188,
    Iron = 100,
    Clay = 12,
}

pub enum TerrainType {
    OneLeft = 0,
    OneHorizontal = 1,
    OneRight = 2,
    OneDown = 3,
    Block = 22,
    TopLeft = 23,
    TopRight = 24,
    OneVertical = 25,
    BottomLeft = 45,
    BottomRight = 46,
    OneUp = 47,
}

impl TerrainMaterial {
    pub fn to_sprite(self, terrain_type: TerrainType) -> usize {
        use TerrainMaterial::*;
        match self {
            Gold | Clay | Copper | Iron | Brick => self as usize + terrain_type as usize,
        }
    }
}

pub trait MapObject: 'static + Send + Sync + std::any::Any {
    fn spawn(&self, animation_data: &Animations, commands: &mut Commands, map_data: &mut MapData);
    fn object_type(&self) -> super::levels::MapObjectType;
    fn serializable(&self) -> bevy::reflect::serde::Serializable;
    fn clone(&self) -> Box<dyn MapObject>;
}

pub fn spawn_map_objects(
    mut commands: Commands,
    mut events: EventReader<MapEvent>,
    terrain: Res<Animations>,
    mut map_data: ResMut<MapData>,
) {
    for event in events.iter() {
        match event {
            MapEvent::Spawn(obj) => {
                let obj = obj.clone();
                obj.spawn(&terrain, &mut commands, &mut map_data);
            }
        }
    }
}

#[derive(Default, Resource)]
pub struct MapData {
    empty: HashSet<IVec2>,
}

impl MapData {
    pub fn is_empty(&self, cell: IVec2) -> bool {
        !self.empty.contains(&cell)
    }

    pub fn set_full(&mut self, cell: IVec2) {
        self.empty.insert(cell);
    }
}
