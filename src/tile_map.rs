use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainSprites>()
            .add_event::<MapEvent>()
            .add_system(spawn_map_objects)
            .init_resource::<MapData>();
    }
}

pub enum MapEvent {
    Spawn(Box<dyn MapObject>),
}

#[derive(Resource)]
pub struct TerrainSprites(Handle<TextureAtlas>);

impl TerrainSprites {
    fn new(handle: Handle<TextureAtlas>) -> TerrainSprites {
        TerrainSprites(handle)
    }
    pub fn get_atlas(&self) -> Handle<TextureAtlas> {
        self.0.clone()
    }
}

impl FromWorld for TerrainSprites {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture_atles = TextureAtlas::from_grid(
            asset_server.load("Terrain/Terrain (16x16).png"),
            Vec2::splat(16.),
            22,
            11,
            None,
            None,
        );
        let mut assets = world.resource_mut::<Assets<TextureAtlas>>();
        TerrainSprites::new(assets.add(texture_atles))
    }
}

#[derive(Clone, Copy)]
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
    fn to_sprite(self, terrain_type: TerrainType) -> usize {
        use TerrainMaterial::*;
        match self {
            Gold | Clay | Copper | Iron | Brick => self as usize + terrain_type as usize,
        }
    }
}

pub trait MapObject: 'static + Send + Sync {
    fn spawn(&self, terrain: &TerrainSprites, commands: &mut Commands, map_data: &mut MapData);
}

pub struct MapBox {
    pub offset: IVec3,
    pub width: i32,
    pub hight: i32,
    pub material: TerrainMaterial,
}

impl MapObject for MapBox {
    fn spawn(&self, terrain: &TerrainSprites, commands: &mut Commands, map_data: &mut MapData) {
        match (self.width, self.hight) {
            (1, 1) => {
                let offset_x = self.offset.x * 16;
                let offset_y = self.offset.y * 16;
                commands.spawn(CellBundle {
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
                    texture_atlas: terrain.get_atlas(),
                    ..Default::default()
                });
            }
            (1, size_y) => {
                let offset_x = self.offset.x as f32 * 16.;
                let offset_y = (self.offset.y as f32 + (size_y as f32 / 2.)) * 16. - 8.;
                commands
                    .spawn((
                        SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(
                                offset_x,
                                offset_y,
                                self.offset.z as f32,
                            )),
                            ..Default::default()
                        },
                        Collider::cuboid(8., size_y as f32 * 8.),
                        RigidBody::Fixed,
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
                                terrain.get_atlas(),
                            ));
                        }
                    });
            }
            (size_x, 1) => {
                let offset_x = (self.offset.x as f32 + (size_x as f32 / 2.)) * 16. - 8.;
                let offset_y = self.offset.y as f32 * 16.;
                commands
                    .spawn((
                        SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(
                                offset_x,
                                offset_y,
                                self.offset.z as f32,
                            )),
                            ..Default::default()
                        },
                        Collider::cuboid(size_x as f32 * 8., 8.),
                        RigidBody::Fixed,
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
                                terrain.get_atlas(),
                            ));
                        }
                    });
            }
            (2, 2) => {
                let offset_x = (self.offset.x + 1) as f32 * 16. - 8.;
                let offset_y = (self.offset.y + 1) as f32 * 16. - 8.;
                commands
                    .spawn((
                        SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(
                                offset_x,
                                offset_y,
                                self.offset.z as f32,
                            )),
                            ..Default::default()
                        },
                        Collider::cuboid(16., 16.),
                        RigidBody::Fixed,
                    ))
                    .add_children(|p| {
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
                            terrain.get_atlas(),
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
                            terrain.get_atlas(),
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
                            terrain.get_atlas(),
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
                            terrain.get_atlas(),
                        ));
                    });
            }
            (x, y) => {
                warn!("Spawning boxes of size ({},{}) is not implmeted", x, y);
            }
        }
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

fn spawn_map_objects(
    mut commands: Commands,
    mut events: EventReader<MapEvent>,
    terrain: Res<TerrainSprites>,
    mut map_data: ResMut<MapData>,
) {
    for event in events.iter() {
        match event {
            MapEvent::Spawn(obj) => {
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
