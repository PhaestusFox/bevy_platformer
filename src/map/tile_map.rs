use std::collections::HashSet;

use crate::{animation::Animations, GameState};
use bevy::{prelude::*, reflect::DynamicTypePath};
use serde::{Deserialize, Serialize};
use bevy_inspector_egui::prelude::*;

#[derive(Event)]
pub enum MapEvent {
    Spawn(Box<dyn MapObject>),
}

impl MapEvent {
    pub fn spawn(object: impl MapObject) -> MapEvent {
        MapEvent::Spawn(Box::new(object))
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, Reflect, Default, Debug, bevy_inspector_egui::InspectorOptions)]
#[reflect(Serialize, InspectorOptions)]
pub enum TerrainMaterial {
    #[default]
    Gold = 193,
    Brick = 105,
    Copper = 188,
    Iron = 100,
    Clay = 12,
}

#[derive(Clone, Copy)]
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

pub trait MapObject: 'static + Send + Sync + std::any::Any + DynamicTypePath {
    fn spawn(
        &self,
        animation_data: &Animations,
        commands: &mut Commands,
        map_data: &mut MapData,
    ) -> Option<Entity>;
    fn object_type(&self) -> super::levels::MapObjectType;
    fn serialize(&self) -> bevy::reflect::serde::Serializable;
    fn clone(&self) -> Box<dyn MapObject>;
    fn set_full(&self, map: &mut MapData);
}

pub(crate) fn spawn_map_objects(
    mut commands: Commands,
    mut events: EventReader<MapEvent>,
    terrain: Res<Animations>,
    mut map_data: ResMut<MapData>,
    state: Res<State<GameState>>,
) {
    let mut last = None;
    for event in events.iter() {
        match event {
            MapEvent::Spawn(obj) => {
                let obj = obj.clone();
                let entity = obj.spawn(&terrain, &mut commands, &mut map_data);
                if entity.is_some() {
                    last = entity;
                }
            }
        }
    }
    if *state.get() == GameState::LevelEditor && last.is_some() {
        commands.insert_resource(crate::editor::LastObj(last));
    }
}

#[derive(Default, Resource)]
pub struct MapData {
    pub(super) need_correcting: bool,
    full: HashSet<IVec2>,
}

impl MapData {
    pub(crate) fn clear(&mut self) {
        self.full.clear();
    }

    pub fn is_empty(&self, cell: IVec2) -> bool {
        !self.full.contains(&cell)
    }

    pub fn set_full(&mut self, cell: IVec2) {
        self.full.insert(cell);
    }

    pub fn shrink(&mut self) {
        self.need_correcting = true;
    }
}
