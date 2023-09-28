mod font_asset;
mod mesh_asset;
mod shader_asset;
mod sprite_asset;
mod texture_asset;

pub use font_asset::*;
pub use mesh_asset::*;
pub use shader_asset::*;
pub use sprite_asset::*;
pub use texture_asset::*;

use std::sync::Arc;

pub type Font = Arc<dyn FontAsset>;
pub type Mesh = Arc<dyn MeshAsset>;
pub type Shader = Arc<dyn ShaderAsset>;
pub type Sprite = Arc<dyn SpriteAsset>;
pub type Texture = Arc<dyn TextureAsset>;
