use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct AnimationLoader;

impl AssetLoader for AnimationLoader {
    fn extensions(&self) -> &[&str] {
        &["san.ron"]
    }
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move { load_antimation(bytes, load_context).await })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimationAsset {
    #[serde(default)]
    pub id: Option<String>,
    pub fps: f32,
    pub tile_size: Vec2,
    pub rows: usize,
    pub columns: usize,
    pub texture_path: String,
}

async fn load_antimation<'de, 'a, 'b>(
    bytes: &'a [u8],
    context: &'a mut bevy::asset::LoadContext<'b>,
) -> Result<(), bevy::asset::Error> {
    let assets: Vec<AnimationAsset> = ron::de::from_bytes(bytes)?;

    for asset_data in assets {
        let image_handle = context.get_handle(&asset_data.texture_path);

        let atlas = TextureAtlas::from_grid(
            image_handle,
            asset_data.tile_size,
            asset_data.columns,
            asset_data.rows,
            None,
            None,
        );
        let loaded_atlas = LoadedAsset::new(atlas).with_dependency(asset_data.texture_path.into());
        if let Some(id) = asset_data.id {
            let atlas = context.set_labeled_asset(&format!("{}_Atlas", id), loaded_atlas);

            let asset = super::SpriteAnimation {
                len: asset_data.rows * asset_data.columns,
                frame_time: 1. / asset_data.fps,
                texture_atlas: atlas,
            };

            context.set_labeled_asset(&id, LoadedAsset::new(asset));
        } else {
            let atlas = context.set_labeled_asset("Atlas", loaded_atlas);

            let asset = super::SpriteAnimation {
                len: asset_data.rows * asset_data.columns,
                frame_time: 1. / asset_data.fps,
                texture_atlas: atlas,
            };
            context.set_default_asset(LoadedAsset::new(asset));
        }
    }
    Ok(())
}

#[test]
fn ron() {
    let animation = AnimationAsset {
        id: Some("Idle".to_string()),
        fps: 20.,
        tile_size: Vec2::splat(32.),
        rows: 5,
        columns: 10,
        texture_path: "SomePath/".to_string(),
    };
    println!(
        "{:?}",
        ron::ser::to_string_pretty(&vec![animation], ron::ser::PrettyConfig::default())
    );
}
