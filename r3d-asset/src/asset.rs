use crate::assets::{Font, Material, Model, Shader, Texture};
use std::{fmt::Display, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Font,
    Material,
    Model,
    Shader,
    Texture,
}

impl Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::Font => write!(f, "font"),
            AssetType::Material => write!(f, "material"),
            AssetType::Model => write!(f, "model"),
            AssetType::Shader => write!(f, "shader"),
            AssetType::Texture => write!(f, "texture"),
        }
    }
}

#[derive(Clone)]
pub enum TypedAsset {
    Font(Font),
    Material(Material),
    Model(Model),
    Shader(Shader),
    Texture(Texture),
}

impl TypedAsset {
    pub fn ty(&self) -> AssetType {
        match self {
            TypedAsset::Font(_) => AssetType::Font,
            TypedAsset::Material(_) => AssetType::Material,
            TypedAsset::Model(_) => AssetType::Model,
            TypedAsset::Shader(_) => AssetType::Shader,
            TypedAsset::Texture(_) => AssetType::Texture,
        }
    }

    pub fn is_font(&self) -> bool {
        matches!(self, TypedAsset::Font(_))
    }

    pub fn is_material(&self) -> bool {
        matches!(self, TypedAsset::Material(_))
    }

    pub fn is_model(&self) -> bool {
        matches!(self, TypedAsset::Model(_))
    }

    pub fn is_shader(&self) -> bool {
        matches!(self, TypedAsset::Shader(_))
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

    pub fn as_material(&self) -> Option<&Material> {
        match self {
            TypedAsset::Material(material) => Some(material),
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
