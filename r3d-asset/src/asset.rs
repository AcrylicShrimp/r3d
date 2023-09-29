use crate::assets::{Font, Model, Shader, Sprite, Texture};
use std::{fmt::Display, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Font,
    Model,
    Shader,
    Sprite,
    Texture,
}

impl Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::Font => write!(f, "font"),
            AssetType::Model => write!(f, "model"),
            AssetType::Shader => write!(f, "shader"),
            AssetType::Sprite => write!(f, "sprite"),
            AssetType::Texture => write!(f, "texture"),
        }
    }
}

#[derive(Clone)]
pub enum TypedAsset {
    Font(Font),
    Model(Model),
    Shader(Shader),
    Sprite(Sprite),
    Texture(Texture),
}

impl TypedAsset {
    pub fn ty(&self) -> AssetType {
        match self {
            TypedAsset::Font(_) => AssetType::Font,
            TypedAsset::Model(_) => AssetType::Model,
            TypedAsset::Shader(_) => AssetType::Shader,
            TypedAsset::Sprite(_) => AssetType::Sprite,
            TypedAsset::Texture(_) => AssetType::Texture,
        }
    }

    pub fn is_font(&self) -> bool {
        matches!(self, TypedAsset::Font(_))
    }

    pub fn is_model(&self) -> bool {
        matches!(self, TypedAsset::Model(_))
    }

    pub fn is_shader(&self) -> bool {
        matches!(self, TypedAsset::Shader(_))
    }

    pub fn is_sprite(&self) -> bool {
        matches!(self, TypedAsset::Sprite(_))
    }

    pub fn is_texture(&self) -> bool {
        matches!(self, TypedAsset::Texture(_))
    }

    pub fn as_font(&self) -> Option<&Font> {
        match self {
            TypedAsset::Font(font) => Some(font),
            _ => None,
        }
    }

    pub fn as_model(&self) -> Option<&Model> {
        match self {
            TypedAsset::Model(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_shader(&self) -> Option<&Shader> {
        match self {
            TypedAsset::Shader(shader) => Some(shader),
            _ => None,
        }
    }

    pub fn as_sprite(&self) -> Option<&Sprite> {
        match self {
            TypedAsset::Sprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    pub fn as_texture(&self) -> Option<&Texture> {
        match self {
            TypedAsset::Texture(texture) => Some(texture),
            _ => None,
        }
    }
}

pub trait Asset {
    fn id(&self) -> Uuid;
    fn as_typed(self: Arc<Self>) -> TypedAsset;
}
