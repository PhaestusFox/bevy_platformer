use bevy::prelude::*;
use rand::Rng;
use crate::animation::{Animations, Animation};

use super::*;

#[derive(Component, Clone)]
pub struct Collectable {
    pub collectable_type: CollectableType,
    pub spawn_type: SpawnType,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Clone)]
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
    fn spawn(&self, terrain: &Animations, commands: &mut Commands, map_data: &mut MapData) {
        let mut new_self = self.clone();
        let mut set_none = false;
        let pos = match &mut new_self.spawn_type {
            SpawnType::None => {return;}
            SpawnType::RandomRange(IVec2 { x: x0, y: y0 }, IVec2 { x: x1, y: y1 }) => {
                let mut rng = rand::thread_rng();
                let x_range = *x0.min(x1)..*x0.max(x1);
                let y_range = *y0.min(y1)..*y0.max(y1);
                let mut trys = 0;
                loop {
                    if trys > MAX_RNG_TRYS {
                        error!("Too many rng trys");
                        return;
                    }
                    trys += 1;
                    let x = rng.gen_range(x_range.clone());
                    let y = rng.gen_range(y_range.clone());
                    if map_data.is_empty(IVec2 { x, y }) {
                        break Vec3::new(x as f32 * 16., y as f32 * 16., 1.);
                    }
                }
            },
            SpawnType::RandomPoints(points) => {
                if points.len() == 0 {
                    error!("No Random points given");
                    return;
                }
                let IVec2 { x, y } = points[rand::thread_rng().gen_range(0..points.len())];
                Vec3::new(x as f32 * 16., y as f32 * 16., 1.)
            },
            SpawnType::Fixed(IVec2 { x, y }) => {
                set_none = true;
                Vec3::new(*x as f32 * 16., *y as f32 * 16., 1.)
            },
            SpawnType::Order(list, index) => {
                if list.len() == 0 {
                    error!("Order Can't Be Empty"); return;
                }
                *index += 1;
                *index %= list.len();
                let IVec2 { x, y } = list[*index];
                Vec3::new(x as f32 * 16., y as f32 * 16., 1.)
            },
            SpawnType::OrderDec(list) => {
                let Some(IVec2{x, y}) = list.pop() else {error!("OrderDec Can't Be Empty"); return;};
                if list.len() == 0 {
                    set_none = true;
                }
                Vec3::new(x as f32 * 16., y as f32 * 16., 1.)
            },
            SpawnType::RandomPointsDec(points) => {
                if points.len() == 0 {
                    error!("RandomPointsDec Can't Be Empty"); return;
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
        let Some(animation) = terrain.get_animation(self.collectable_type.into()) else {error!("Animation for {:?} not loaded", self.collectable_type); return;};
        
        commands.spawn((
            SpriteSheetBundle {
                transform: Transform::from_translation(pos),
                texture_atlas: default(),
                ..Default::default()
            },
            animation,
            RigidBody::Fixed,
            Collider::ball(8.),
            Sensor,
            Name::new("Collectable"),
            new_self,
        ));
    }
}