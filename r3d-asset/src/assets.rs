mod font_asset;
mod model_asset;
mod shader_asset;
mod texture_asset;

pub use font_asset::*;
pub use model_asset::*;
pub use shader_asset::*;
pub use texture_asset::*;

use std::sync::Arc;

pub type Font = Arc<dyn FontAsset>;
pub type Model = Arc<dyn ModelAsset>;
pub type Shader = Arc<dyn ShaderAsset>;
pub type Texture = Arc<dyn TextureAsset>;
