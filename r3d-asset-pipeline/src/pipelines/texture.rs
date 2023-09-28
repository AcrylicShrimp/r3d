use crate::{AssetPipeline, Metadata};
use asset::assets::{TextureAddressMode, TextureFilterMode, TextureFormat, TextureSource};
use image::io::Reader as ImageReader;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Serialize, Deserialize)]
pub struct TextureMetadata {
    pub texture: TextureTable,
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

impl AssetPipeline for TextureSource {
    type Metadata = TextureMetadata;

    fn process(file_content: Vec<u8>, metadata: &Metadata<Self::Metadata>) -> anyhow::Result<Self> {
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
        Ok(Self {
            width,
            height,
            format,
            filter_mode,
            address_mode,
            texels,
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
