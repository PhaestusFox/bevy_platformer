use super::*;
use crate::animation::{Animation, Animations};
use belly::{prelude::*, build::{widget, FromWorldAndParams, Variant}, core::{eml::FromWorldAndParam, relations::bind::ComponentToComponent}};
use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, Reflect, Component, Default)]
#[reflect(Component)]
pub struct Square {
    pub offset: IVec3,
    pub size: IVec2,
    pub material: TerrainMaterial,
}

impl MapObject for Square {
    fn spawn(
        &self,
        terrain: &Animations,
        commands: &mut Commands,
        map_data: &mut MapData,
    ) -> Option<Entity> {
        Some({
            // map_data.need_correcting = true;
            self.set_full(map_data);
            commands
                    .spawn((
                        CellBundle {
                            transform: Transform::from_translation(calc_pos(self.offset, self.size)),
                            collider: Collider::cuboid(8., 8.),
                            rigid_body: RigidBody::Fixed,
                            item: *self,
                            ..Default::default()
                        },
                    ))
                    .id()}
        )
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
    fn set_full(&self, map: &mut MapData) {
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                map.set_full(IVec2::new(x + self.offset.x, y + self.offset.y));
            }
        }
    }
}

impl DrawProps for Square {
    fn ui_draw(self_entity: Entity) -> belly::core::eml::Eml {
        eml! {
            <button entity=self_entity c:icon on:press=run!(|ctx| {
                ctx.commands().add(|world: &mut World| {
                    world.send_event(MapEvent::Spawn(Box::new(Square {
                        offset: IVec3::default(),
                        size: IVec2::splat(1),
                        material: TerrainMaterial::Brick,
                    })));
                });
            })>
            <img src="Icons/block.png"/>
            </button>
        }
    }
    fn draw_props(root: Entity) -> belly::core::eml::Eml {
        eml!(<square root=root/>)
    }
}

#[widget]
fn square(ctx: &mut belly::build::WidgetContext) {
    let Some(root) = ctx.required_param::<Entity>("root") else { return };
    ctx.render(eml! {
        <div id="editor">
        <label value="I am a box"/>
        <button on:press=run!(for root |data: &mut Square| {
            data.size.x -= 1;
                    })>"-"</button>
        <label value="width"/>
        <button on:press=run!(for root |data: &mut Square| {
            data.size.x += 1;
                    })>"+"</button>
        <button on:press=run!(for root |data: &mut Square| {
data.material = TerrainMaterial::Gold;
        })>"="</button>
        <label bind:value=from!(root, Square:material|fmt.c("{:?}", c))/>
        </div>
    });
}

pub fn update_square(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Square, &mut Collider), Changed<Square>>,
    terrain: Res<Animations>,
) {
    for (entity, mut transform, square, mut collider) in &mut query {
        commands.entity(entity).despawn_descendants();
        spawn_square_sprites(&mut commands, entity, square.size, terrain.get_atlas(Animation::Terrain).expect("Fuck You"));
        transform.translation = calc_pos(square.offset, square.size);
        *collider = Collider::cuboid(square.size.x as f32 * 8., square.size.y as f32 * 8.);
    }
}

fn calc_pos(offset: IVec3, size: IVec2) -> Vec3 {
    match (size.x, size.y) {
        (1, 1) => (offset * 16).as_vec3(),
        (1, size_y) => Vec3::new(
            (offset.x * 16) as f32,
            (offset.y as f32 + (size_y as f32 / 2.)) * 16. - 8.,
            (offset.z * 16) as f32),
        (size_x, 1) => Vec3::new(
            (offset.x as f32 + (size_x as f32 / 2.)) * 16. - 8.,
            (offset.y * 16) as f32,
            (offset.z * 16) as f32),
        (x, y) => Vec3::new(
            (offset.x as f32 + (x as f32 / 2.)) * 16. - 8.,
            (offset.y as f32 + (y as f32 / 2.)) * 16. - 8.,
            (offset.z * 16) as f32),
    }
}

fn spawn_square_sprites(
    commands: &mut Commands,
    parent: Entity,
    size: IVec2,
    terrain: Handle<TextureAtlas>,
) {
            match (size.x, size.y) {
            (1, 1) => {
                commands.entity(parent).with_children(|p| {
                            let child = p.spawn((
                                SpatialBundle::default(),
                                TextureAtlasSprite::default(),
                                terrain
                            )
                            ).id();
                            p.add_command(from!(parent, Square:material.to_sprite(TerrainType::Block)) >> to!(child, TextureAtlasSprite:index));
                        }
                    );
            }
            (1, size_y) => {
                    commands.entity(parent)
                    .with_children(|p| {
                        let range = if size_y % 2 == 1 {
                            -size_y / 2..=size_y / 2
                        } else {
                            (-size_y / 2 + 1)..=size_y / 2
                        };
                        for (i, y) in range.enumerate() {
                            let terrain_type = if i == 0 {
                                TerrainType::OneUp
                            } else if i == size.y as usize - 1 {
                                TerrainType::OneDown
                            } else {
                                TerrainType::OneVertical
                            };
                            let child = p.spawn((
                                SpatialBundle {
                                    transform: if size_y % 2 == 0 {
                                        Transform::from_translation(Vec3::Y * (y as f32 * 16. - 8.))
                                    } else {
                                        Transform::from_translation(Vec3::Y * (y as f32 * 16.))
                                    },
                                    ..Default::default()
                                },
                                TextureAtlasSprite::default(),
                                terrain.clone()
                                )).id();
                                p.add_command(bind_to(parent, child, terrain_type));
                        }
                    });
            }
            (size_x, 1) => {
                commands.entity(parent).with_children(|p| {
                        let range = if size_x % 2 == 1 {
                            -size_x / 2..=size_x / 2
                        } else {
                            (-size_x / 2 + 1)..=size_x / 2
                        };
                        for (i, x) in range.enumerate() {
                            let terrain_type = if i == 0 {
                                TerrainType::OneLeft
                            } else if i == size.x as usize - 1 {
                                TerrainType::OneRight
                            } else {
                                TerrainType::OneHorizontal
                            };
                            let child = p.spawn((
                                SpatialBundle {
                                    transform: if size_x % 2 == 0 {
                                        Transform::from_translation(Vec3::X * (x as f32 * 16. - 8.))
                                    } else {
                                        Transform::from_translation(Vec3::X * (x as f32 * 16.))
                                    },
                                    ..Default::default()
                                },
                                TextureAtlasSprite::default(),
                                terrain.clone(),
                            )).id();
                            p.add_command(bind_to(parent, child, terrain_type));
                        }
                    });
            }
            (2, 2) => {
                    commands.entity(parent)
                    .with_children(|p| {
                        let child = p.spawn((
                            SpatialBundle {
                                transform: Transform::from_translation(Vec3::new(-8., -8., 0.)),
                                ..Default::default()
                            },
                            TextureAtlasSprite::default(),
                            terrain.clone(),
                        )).id();
                        p.add_command(from!(parent, Square:material.to_sprite(TerrainType::BottomLeft)) >> to!(child, TextureAtlasSprite:index));
                        let child = p.spawn((
                            SpatialBundle {
                                transform: Transform::from_translation(Vec3::new(8., -8., 0.)),
                                ..Default::default()
                            },
                            TextureAtlasSprite::default(),
                            terrain.clone(),
                        )).id();
                        p.add_command(from!(parent, Square:material.to_sprite(TerrainType::BottomRight)) >> to!(child, TextureAtlasSprite:index));
                        let child = p.spawn((
                            SpatialBundle {
                                transform: Transform::from_translation(Vec3::new(-8., 8., 0.)),
                                ..Default::default()
                            },
                            TextureAtlasSprite::default(),
                            terrain.clone(),
                        )).id();
                        p.add_command(from!(parent, Square:material.to_sprite(TerrainType::TopLeft)) >> to!(child, TextureAtlasSprite:index));
                        let child = p.spawn((
                            SpatialBundle {
                                transform: Transform::from_translation(Vec3::new(8., 8., 0.)),
                                ..Default::default()
                            },
                            TextureAtlasSprite::default(),
                            terrain.clone(),
                        )).id();
                        p.add_command(from!(parent, Square:material.to_sprite(TerrainType::TopRight)) >> to!(child, TextureAtlasSprite:index));
                    });
            },
            (x, y) => {
                error!("square({}, {}) not impl", x, y);
            }
        }
}

fn bind_to(from: Entity, to: Entity, terrain: TerrainType) -> ComponentToComponent<Square, TextureAtlasSprite, usize, usize> {
    match terrain {
        TerrainType::OneLeft => from!(from, Square:material.to_sprite(TerrainType::OneLeft)) >> to!(to, TextureAtlasSprite:index),
        TerrainType::OneHorizontal => from!(from, Square:material.to_sprite(TerrainType::OneHorizontal)) >> to!(to, TextureAtlasSprite:index),
        TerrainType::OneRight => from!(from, Square:material.to_sprite(TerrainType::OneRight)) >> to!(to, TextureAtlasSprite:index),
        TerrainType::OneDown => from!(from, Square:material.to_sprite(TerrainType::OneDown)) >> to!(to, TextureAtlasSprite:index),
        TerrainType::Block => from!(from, Square:material.to_sprite(TerrainType::Block)) >> to!(to, TextureAtlasSprite:index),
        TerrainType::TopLeft => from!(from, Square:material.to_sprite(TerrainType::TopLeft)) >> to!(to, TextureAtlasSprite:index),
        TerrainType::TopRight => from!(from, Square:material.to_sprite(TerrainType::TopRight)) >> to!(to, TextureAtlasSprite:index),
        TerrainType::OneVertical => from!(from, Square:material.to_sprite(TerrainType::OneVertical)) >> to!(to, TextureAtlasSprite:index),
        TerrainType::BottomLeft => from!(from, Square:material.to_sprite(TerrainType::BottomLeft)) >> to!(to, TextureAtlasSprite:index),
        TerrainType::BottomRight => from!(from, Square:material.to_sprite(TerrainType::BottomRight)) >> to!(to, TextureAtlasSprite:index),
        TerrainType::OneUp => from!(from, Square:material.to_sprite(TerrainType::OneUp)) >> to!(to, TextureAtlasSprite:index),
    }
}

pub fn update_map_with_squares(
    query: Query<&Square>,
    mut map: ResMut<MapData>,
) {
    println!("update squares");
    for square in &query {
        square.set_full(&mut map);
    }
}