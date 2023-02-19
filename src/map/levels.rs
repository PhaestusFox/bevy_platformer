use bevy::{prelude::*, reflect::TypeUuid, asset::{AssetLoader, LoadedAsset}};
use serde::{Serialize, Deserialize, de::{Visitor, DeserializeSeed}};

use super::*;

#[derive(TypeUuid)]
#[uuid = "e6b53f1c-9471-465c-b411-7729177acb9e"]
pub struct Level {
    pub player_start: IVec2,
    pub objects: Vec<Box<dyn MapObject>>,
}

impl<'de> Deserialize<'de> for Level {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_struct("Level", &["player_start", "objects"], LevelVisitor)
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
pub enum LevelFields {
    Start,
    Objects,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MapObjectType {
    Box,
    Collectable,
}

struct LevelVisitor;

impl<'de> Visitor<'de> for LevelVisitor {
    type Value = Level;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected Level Data")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        let mut data = Level { player_start: IVec2::ZERO, objects: Vec::new() };
        while let Some(key) = map.next_key::<LevelFields>()? {
            match key {
                LevelFields::Start => {
                    data.player_start = map.next_value::<IVec2>()?;
                },
                LevelFields::Objects => {
                    data.objects = map.next_value_seed(ObjectsVisitor)?;
                },
            }
        }
        Ok(data)
    }
}

struct ObjectsVisitor;

impl<'de> DeserializeSeed<'de> for ObjectsVisitor {
    type Value = <Self as Visitor<'de>>::Value;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for ObjectsVisitor{
    type Value = Vec<Box<dyn MapObject>>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expecing Map of MapObjects")
    }
    fn visit_map<A>(self,mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        let mut objects: Vec<Box<dyn MapObject>> = Vec::new();
        while let Some(key) = map.next_key::<MapObjectType>()? {
            match key {
                MapObjectType::Box => {
                    objects.push(Box::new(map.next_value::<MapBox>()?));
                },
                MapObjectType::Collectable => {
                    objects.push(Box::new(map.next_value::<Collectable>()?));
                },
            }
        }
        Ok(objects)
    }
}

pub struct LevelLoader;

impl AssetLoader for LevelLoader {
    fn extensions(&self) -> &[&str] {
        &["lvl", "lvl.ron"]
    }
    fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            load_context: &'a mut bevy::asset::LoadContext,
        ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {load_level(bytes, load_context).await})
    }
}

async fn load_level<'a>(
    bytes: &[u8],
    load_context: &mut bevy::asset::LoadContext<'a>) -> Result<(), bevy::asset::Error> {
        let level = ron::de::from_bytes::<Level>(bytes).map_err(|e| bevy::asset::Error::new(e))?;
        load_context.set_default_asset(LoadedAsset::new(level));
        Ok(())
}

impl Serialize for Level {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
                use::serde::ser::SerializeStruct;
        let mut struct_data = serializer.serialize_struct("Level", 2)?;
        struct_data.serialize_field("start", &self.player_start)?;
        struct_data.serialize_field("objects", &ObjectsSerializer(&self.objects))?;
        struct_data.end()
    }
}
struct ObjectsSerializer<'a>(&'a Vec<Box<dyn MapObject>>);

impl<'a> Serialize for ObjectsSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for item in self.0 {
            let item_data = item.serializable(); // else {error!("Can not reflect serializable"); continue;};
            match item_data {
                bevy::reflect::serde::Serializable::Owned(v) => map.serialize_entry(&item.object_type(), &v)?,
                bevy::reflect::serde::Serializable::Borrowed(v) => map.serialize_entry(&item.object_type(), v)?,
            };
        }
        map.end()
    }
}

#[test]
fn test_serialize_level() {
    let level = Level {
        player_start: IVec2::new(0, 0),
        objects: vec![
            Box::new(MapBox{
                offset: IVec3 { x: 10, y: 4, z: 0 },
                width: 1,
                hight: 1,
                material: TerrainMaterial::Gold,
            }),
            Box::new(Collectable{
                collectable_type: CollectableType::Strawberry,
                spawn_type: SpawnType::Fixed(IVec2 { x: 5, y: 5 }),
            })
        ]
    };
    assert_eq!(include_str!("test.lvl.ron"), ron::ser::to_string_pretty(&level, ron::ser::PrettyConfig::default()).unwrap());
}