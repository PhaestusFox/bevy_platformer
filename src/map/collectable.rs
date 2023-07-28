use super::*;
use crate::animation::{Animation, Animations};
use bevy::{ecs::system::EntityCommands, prelude::*};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Deserialize, Serialize, Reflect)]
pub struct Collectable {
    pub collectable_type: CollectableType,
    pub spawn_type: SpawnType,
}

impl Default for Collectable {
    fn default() -> Self {
        Collectable { collectable_type: CollectableType::Strawberry, spawn_type: SpawnType::None }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Reflect)]
#[reflect_value()]
pub enum CollectableType {
    Strawberry,
    Bananan,
}

impl Into<Animation> for CollectableType {
    fn into(self) -> Animation {
        match self {
            CollectableType::Strawberry => Animation::Strawberry,
            CollectableType::Bananan => Animation::Bananas,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Reflect)]
#[reflect_value()]
pub enum SpawnType {
    None,
    RandomRange(IVec2, IVec2),
    RandomPoints(Vec<IVec2>),
    Fixed(IVec2),
    Order(Vec<IVec2>, usize),
    OrderDec(Vec<IVec2>),
    RandomPointsDec(Vec<IVec2>),
}
const MAX_RNG_TRYS: usize = 50;

impl MapObject for Collectable {
    fn spawn(
        &self,
        terrain: &Animations,
        commands: &mut Commands,
        map_data: &mut MapData,
    ) -> Option<Entity> {
        let mut new_self = <Self as Clone>::clone(self);
        let mut set_none = false;
        let pos = match &mut new_self.spawn_type {
            SpawnType::None => {
                return None;
            }
            SpawnType::RandomRange(IVec2 { x: x0, y: y0 }, IVec2 { x: x1, y: y1 }) => {
                let mut rng = rand::thread_rng();
                let x_range = *x0.min(x1)..*x0.max(x1);
                let y_range = *y0.min(y1)..*y0.max(y1);
                let mut trys = 0;
                loop {
                    if trys > MAX_RNG_TRYS {
                        error!("Too many rng trys");
                        return None;
                    }
                    trys += 1;
                    let x = rng.gen_range(x_range.clone());
                    let y = rng.gen_range(y_range.clone());
                    if map_data.is_empty(IVec2 { x, y }) {
                        break Vec3::new(x as f32 * 16., y as f32 * 16., 1.);
                    }
                }
            }
            SpawnType::RandomPoints(points) => {
                if points.len() == 0 {
                    error!("No Random points given");
                    return None;
                }
                let IVec2 { x, y } = points[rand::thread_rng().gen_range(0..points.len())];
                Vec3::new(x as f32 * 16., y as f32 * 16., 1.)
            }
            SpawnType::Fixed(IVec2 { x, y }) => {
                set_none = true;
                Vec3::new(*x as f32 * 16., *y as f32 * 16., 1.)
            }
            SpawnType::Order(list, index) => {
                if list.len() == 0 {
                    error!("Order Can't Be Empty");
                    return None;
                }
                *index += 1;
                *index %= list.len();
                let IVec2 { x, y } = list[*index];
                Vec3::new(x as f32 * 16., y as f32 * 16., 1.)
            }
            SpawnType::OrderDec(list) => {
                let Some(IVec2{x, y}) = list.pop() else {error!("OrderDec Can't Be Empty"); return None;};
                if list.len() == 0 {
                    set_none = true;
                }
                Vec3::new(x as f32 * 16., y as f32 * 16., 1.)
            }
            SpawnType::RandomPointsDec(points) => {
                if points.len() == 0 {
                    error!("RandomPointsDec Can't Be Empty");
                    return None;
                } else if points.len() == 1 {
                    set_none = true;
                }
                let index = rand::thread_rng().gen_range(0..points.len());
                let IVec2 { x, y } = points.remove(index);
                Vec3::new(x as f32 * 16., y as f32 * 16., 1.)
            }
        };
        // cant update a mutable refence while borrowed
        // this sets next spawn to none on collectables that dont move
        if set_none {
            new_self.spawn_type = SpawnType::None;
        }
        let Some(animation) = terrain.get_animation(self.collectable_type.into()) else {error!("Animation for {:?} not loaded", self.collectable_type); return None;};

        Some(
            commands
                .spawn((
                    CellBundle {
                        transform: Transform::from_translation(pos),
                        rigid_body: RigidBody::Fixed,
                        collider: Collider::ball(8.),
                        item: new_self,
                        ..Default::default()
                    },

                    Handle::<TextureAtlas>::default(),
                    TextureAtlasSprite::default(),
                    animation,
                    Sensor,
                    Name::new("Collectable"),
                ))
                .id(),
        )
    }
    fn object_type(&self) -> super::levels::MapObjectType {
        super::levels::MapObjectType::Collectable
    }
    fn serialize(&self) -> bevy::reflect::serde::Serializable {
        bevy::reflect::serde::Serializable::Borrowed(self)
    }
    fn clone(&self) -> Box<dyn MapObject> {
        Box::new(<Self as Clone>::clone(self))
    }
    fn set_full(&self, _: &mut MapData) {}
}

impl DrawProps for Collectable {
    fn draw_props(_root: Entity) -> belly::core::eml::Eml {
        use belly::prelude::*;
        eml!(<label {_root} value="Collectable Not done"/>)
    }
    fn ui_draw(_editor: Entity) -> belly::core::eml::Eml {
        use belly::prelude::*;
        eml!(<label {_editor} value="Collectable Not done"/>)
    }
}