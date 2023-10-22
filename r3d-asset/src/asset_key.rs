use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetKey {
    Id(Uuid),
    Path(String),
}

impl Display for AssetKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetKey::Id(id) => {
                write!(f, "[asset id={}]", id)
            }
            AssetKey::Path(path) => {
                write!(f, "[asset path={}]", path)
            }
        }
    }
}
