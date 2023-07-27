use super::*;
use crate::animation::{Animation, Animations};
use bevy::{prelude::*, ecs::system::EntityCommands};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, Reflect, Component)]
pub struct MapBox {
    pub offset: IVec3,
    pub width: i32,
    pub hight: i32,
    pub material: TerrainMaterial,
}

impl MapObject for MapBox {
    fn spawn(
        &self,
        terrain: &Animations,
        commands: &mut Commands,
        map_data: &mut MapData,
    ) -> Option<Entity> {
        Some(match (self.width, self.hight) {
            (1, 1) => {
                let offset_x = self.offset.x * 16;
                let offset_y = self.offset.y * 16;
                commands
                    .spawn((
                        CellBundle {
                            transform: Transform::from_translation(Vec3::new(
                                offset_x as f32,
                                offset_y as f32,
                                self.offset.z as f32,
                            )),
                            collider: Collider::cuboid(8., 8.),
                            rigid_body: RigidBody::Fixed,
                            sprite: TextureAtlasSprite {
                                index: self.material.to_sprite(TerrainType::Block) as usize,
                                ..Default::default()
                            },
                            texture_atlas: terrain
                                .get_atlas(Animation::Terrain)
                                .expect("Terrain is loaded"),
                            ..Default::default()
                        },
                        *self,
                    ))
                    .id()
            }
            (1, size_y) => {
                let offset_x = self.offset.x as f32 * 16.;
                let offset_y = (self.offset.y as f32 + (size_y as f32 / 2.)) * 16. - 8.;
                commands
                    .spawn((
                        CellBundle {
                            transform: Transform::from_translation(Vec3::new(
                                offset_x,
                                offset_y,
                                self.offset.z as f32,
                            )),
                            collider: Collider::cuboid(8., size_y as f32 * 8.),
                            rigid_body: RigidBody::Fixed,
                            ..Default::default()
                        },
                        *self,
                    ))
                    .with_children(|p| {
                        let range = if size_y % 2 == 1 {
                            -size_y / 2..=size_y / 2
                        } else {
                            (-size_y / 2 + 1)..=size_y / 2
                        };
                        for (i, y) in range.enumerate() {
                            map_data.set_full(IVec2::new(self.offset.x, self.offset.y + y));
                            p.spawn((
                                SpatialBundle {
                                    transform: if size_y % 2 == 0 {
                                        Transform::from_translation(Vec3::Y * (y as f32 * 16. - 8.))
                                    } else {
                                        Transform::from_translation(Vec3::Y * (y as f32 * 16.))
                                    },
                                    ..Default::default()
                                },
                                TextureAtlasSprite {
                                    index: if i == 0 {
                                        self.material.to_sprite(TerrainType::OneUp) as usize
                                    } else if i == self.hight as usize - 1 {
                                        self.material.to_sprite(TerrainType::OneDown) as usize
                                    } else {
                                        self.material.to_sprite(TerrainType::OneVertical) as usize
                                    },
                                    ..Default::default()
                                },
                                terrain
                                    .get_atlas(Animation::Terrain)
                                    .expect("Terrain is loaded"),
                            ));
                        }
                    })
                    .id()
            }
            (size_x, 1) => {
                let offset_x = (self.offset.x as f32 + (size_x as f32 / 2.)) * 16. - 8.;
                let offset_y = self.offset.y as f32 * 16.;
                commands
                    .spawn((
                        CellBundle {
                            transform: Transform::from_translation(Vec3::new(
                                offset_x,
                                offset_y,
                                self.offset.z as f32,
                            )),
                            collider: Collider::cuboid(size_x as f32 * 8., 8.),
                            rigid_body: RigidBody::Fixed,
                            ..Default::default()
                        },
                        *self,
                    ))
                    .with_children(|p| {
                        let range = if size_x % 2 == 1 {
                            -size_x / 2..=size_x / 2
                        } else {
                            (-size_x / 2 + 1)..=size_x / 2
                        };
                        for (i, x) in range.enumerate() {
                            map_data.set_full(IVec2::new(self.offset.x + x, self.offset.y));
                            p.spawn((
                                SpatialBundle {
                                    transform: if size_x % 2 == 0 {
                                        Transform::from_translation(Vec3::X * (x as f32 * 16. - 8.))
                                    } else {
                                        Transform::from_translation(Vec3::X * (x as f32 * 16.))
                                    },
                                    ..Default::default()
                                },
                                TextureAtlasSprite {
                                    index: if i == 0 {
                                        self.material.to_sprite(TerrainType::OneLeft) as usize
                                    } else if i == self.width as usize - 1 {
                                        self.material.to_sprite(TerrainType::OneRight) as usize
                                    } else {
                                        self.material.to_sprite(TerrainType::OneHorizontal) as usize
                                    },
                                    ..Default::default()
                                },
                                terrain
                                    .get_atlas(Animation::Terrain)
                                    .expect("Terrain is loaded"),
                            ));
                        }
                    })
                    .id()
            }
            (2, 2) => {
                let offset_x = (self.offset.x + 1) as f32 * 16. - 8.;
                let offset_y = (self.offset.y + 1) as f32 * 16. - 8.;
                commands
                    .spawn((
                        CellBundle {
                            transform: Transform::from_translation(Vec3::new(
                                offset_x,
                                offset_y,
                                self.offset.z as f32,
                            )),
                            collider: Collider::cuboid(16., 16.),
                            rigid_body: RigidBody::Fixed,
                            ..Default::default()
                        },
                        *self,
                    ))
                    .with_children(|p| {
                        map_data.set_full(IVec2::new(self.offset.x, self.offset.y));
                        p.spawn((
                            SpatialBundle {
                                transform: Transform::from_translation(Vec3::new(-8., -8., 0.)),
                                ..Default::default()
                            },
                            TextureAtlasSprite {
                                index: self.material.to_sprite(TerrainType::BottomLeft),
                                ..Default::default()
                            },
                            terrain
                                .get_atlas(Animation::Terrain)
                                .expect("Terrain is loaded"),
                        ));
                        map_data.set_full(IVec2::new(self.offset.x + 1, self.offset.y));
                        p.spawn((
                            SpatialBundle {
                                transform: Transform::from_translation(Vec3::new(8., -8., 0.)),
                                ..Default::default()
                            },
                            TextureAtlasSprite {
                                index: self.material.to_sprite(TerrainType::BottomRight),
                                ..Default::default()
                            },
                            terrain
                                .get_atlas(Animation::Terrain)
                                .expect("Terrain is loaded"),
                        ));
                        map_data.set_full(IVec2::new(self.offset.x, self.offset.y + 1));
                        p.spawn((
                            SpatialBundle {
                                transform: Transform::from_translation(Vec3::new(-8., 8., 0.)),
                                ..Default::default()
                            },
                            TextureAtlasSprite {
                                index: self.material.to_sprite(TerrainType::TopLeft),
                                ..Default::default()
                            },
                            terrain
                                .get_atlas(Animation::Terrain)
                                .expect("Terrain is loaded"),
                        ));
                        map_data.set_full(IVec2::new(self.offset.x + 1, self.offset.y + 1));
                        p.spawn((
                            SpatialBundle {
                                transform: Transform::from_translation(Vec3::new(8., 8., 0.)),
                                ..Default::default()
                            },
                            TextureAtlasSprite {
                                index: self.material.to_sprite(TerrainType::TopRight),
                                ..Default::default()
                            },
                            terrain
                                .get_atlas(Animation::Terrain)
                                .expect("Terrain is loaded"),
                        ));
                    })
                    .id()
            }
            (x, y) => {
                warn!("Spawning boxes of size ({},{}) is not implmeted", x, y);
                return None;
            }
        })
    }
    fn object_type(&self) -> super::levels::MapObjectType {
        super::levels::MapObjectType::Box
    }
    fn serialize(&self) -> bevy::reflect::serde::Serializable {
        bevy::reflect::serde::Serializable::Borrowed(self)
    }
    fn clone(&self) -> Box<dyn MapObject> {
        Box::new(<Self as Clone>::clone(self))
    }
    fn ui_draw(&self, commands: EntityCommands) {
        todo!()
    }
}
