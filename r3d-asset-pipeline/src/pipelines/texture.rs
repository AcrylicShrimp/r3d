use crate::{AssetPipeline, Metadata, PipelineGfxBridge};
use asset::assets::{
    NinePatchSource, NinePatchTexelRange, SpriteSource, SpriteTexelRange, TextureAddressMode,
    TextureFilterMode, TextureFormat, TextureSource,
};
use image::io::Reader as ImageReader;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Cursor};

#[derive(Serialize, Deserialize)]
pub struct TextureMetadata {
    pub texture: TextureTable,
    pub sprite: HashMap<String, SpriteTable>,
    pub nine_patch: HashMap<String, NinePatchTable>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TextureTableFilterMode {
    Point,
    Bilinear,
    Trilinear,
}

impl From<TextureTableFilterMode> for TextureFilterMode {
    fn from(value: TextureTableFilterMode) -> Self {
        match value {
            TextureTableFilterMode::Point => Self::Point,
            TextureTableFilterMode::Bilinear => Self::Bilinear,
            TextureTableFilterMode::Trilinear => Self::Trilinear,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TextureTableAddressMode {
    Clamp,
    Repeat,
}

impl From<TextureTableAddressMode> for TextureAddressMode {
    fn from(value: TextureTableAddressMode) -> Self {
        match value {
            TextureTableAddressMode::Clamp => Self::Clamp,
            TextureTableAddressMode::Repeat => Self::Repeat,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TextureTable {
    pub is_srgb: bool,
    pub has_alpha: bool,
    pub filter_mode: TextureTableFilterMode,
    pub address_mode_u: TextureTableAddressMode,
    pub address_mode_v: TextureTableAddressMode,
}

#[derive(Serialize, Deserialize)]
pub struct SpriteTable {
    pub x_min: u16,
    pub x_max: u16,
    pub y_min: u16,
    pub y_max: u16,
    pub filter_mode: Option<TextureTableFilterMode>,
    pub address_mode_u: Option<TextureTableAddressMode>,
    pub address_mode_v: Option<TextureTableAddressMode>,
}

#[derive(Serialize, Deserialize)]
pub struct NinePatchTable {
    pub x_min: u16,
    pub x_mid_min: u16,
    pub x_mid_max: u16,
    pub x_max: u16,
    pub y_min: u16,
    pub y_mid_min: u16,
    pub y_mid_max: u16,
    pub y_max: u16,
    pub filter_mode: Option<TextureTableFilterMode>,
    pub address_mode_u: Option<TextureTableAddressMode>,
    pub address_mode_v: Option<TextureTableAddressMode>,
}

impl AssetPipeline for TextureSource {
    type Metadata = TextureMetadata;

    fn process(
        file_content: Vec<u8>,
        metadata: &Metadata<Self::Metadata>,
        _gfx_bridge: &dyn PipelineGfxBridge,
    ) -> anyhow::Result<Self> {
        let image = ImageReader::new(Cursor::new(file_content))
            .with_guessed_format()?
            .decode()?;
        let width = image.width() as u16;
        let height = image.height() as u16;
        let texels = if metadata.extra.texture.has_alpha {
            let mut image = {
                let rgba = image.to_rgba8();
                drop(image);
                rgba
            };

            if metadata.extra.texture.is_srgb {
                for pixel in image.pixels_mut() {
                    let (r, g, b) = srgb_to_linear(pixel[0], pixel[1], pixel[2]);
                    pixel[0] = r;
                    pixel[1] = g;
                    pixel[2] = b;
                }
            }

            image.into_raw()
        } else {
            let mut image = {
                let rgb = image.to_rgb8();
                drop(image);
                rgb
            };

            if metadata.extra.texture.is_srgb {
                for pixel in image.pixels_mut() {
                    let (r, g, b) = srgb_to_linear(pixel[0], pixel[1], pixel[2]);
                    pixel[0] = r;
                    pixel[1] = g;
                    pixel[2] = b;
                }
            }

            image.into_raw()
        };
        let format = if metadata.extra.texture.has_alpha {
            TextureFormat::RGBA8
        } else {
            TextureFormat::RGB8
        };
        let filter_mode = metadata.extra.texture.filter_mode.into();
        let address_mode = (
            metadata.extra.texture.address_mode_u.into(),
            metadata.extra.texture.address_mode_v.into(),
        );

        let sprites = Vec::from_iter(metadata.extra.sprite.iter().map(|(name, sprite)| {
            SpriteSource {
                name: name.clone(),
                filter_mode: sprite
                    .filter_mode
                    .map(|mode| mode.into())
                    .unwrap_or(filter_mode),
                address_mode: (
                    sprite
                        .address_mode_u
                        .map(|mode| mode.into())
                        .unwrap_or(address_mode.0),
                    sprite
                        .address_mode_v
                        .map(|mode| mode.into())
                        .unwrap_or(address_mode.1),
                ),
                texel_mapping: (
                    SpriteTexelRange {
                        min: sprite.x_min,
                        max: sprite.x_max,
                    },
                    SpriteTexelRange {
                        min: sprite.y_min,
                        max: sprite.y_max,
                    },
                ),
            }
        }));
        let nine_patches =
            Vec::from_iter(metadata.extra.nine_patch.iter().map(|(name, nine_patch)| {
                NinePatchSource {
                    name: name.clone(),
                    filter_mode: nine_patch
                        .filter_mode
                        .map(|mode| mode.into())
                        .unwrap_or(filter_mode),
                    address_mode: (
                        nine_patch
                            .address_mode_u
                            .map(|mode| mode.into())
                            .unwrap_or(address_mode.0),
                        nine_patch
                            .address_mode_v
                            .map(|mode| mode.into())
                            .unwrap_or(address_mode.1),
                    ),
                    texel_mapping: (
                        NinePatchTexelRange {
                            min: nine_patch.x_min,
                            mid_min: nine_patch.x_mid_min,
                            mid_max: nine_patch.x_mid_max,
                            max: nine_patch.x_max,
                        },
                        NinePatchTexelRange {
                            min: nine_patch.y_min,
                            mid_min: nine_patch.y_mid_min,
                            mid_max: nine_patch.y_mid_max,
                            max: nine_patch.y_max,
                        },
                    ),
                }
            }));

        Ok(Self {
            width,
            height,
            format,
            filter_mode,
            address_mode,
            texels,
            sprites,
            nine_patches,
        })
    }
}

fn srgb_to_linear(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    (
        (srgb_to_linear_single(r as f32 / 255.0) * 255.0) as u8,
        (srgb_to_linear_single(g as f32 / 255.0) * 255.0) as u8,
        (srgb_to_linear_single(b as f32 / 255.0) * 255.0) as u8,
    )
}

fn srgb_to_linear_single(channel: f32) -> f32 {
    if channel <= 0.04045f32 {
        channel / 12.92f32
    } else {
        ((channel + 0.055f32) / 1.055f32).powf(2.4f32)
    }
}
