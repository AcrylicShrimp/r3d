use asset::AssetType;
use asset_pipeline::{deduce_asset_type_from_path, AssetMetadata};
use std::{
    collections::HashMap,
    hash::Hash,
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AssetDatabaseError {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("toml error: {0}")]
    TOMLError(#[from] toml::de::Error),
    #[error("failed to deduce asset type: {0}")]
    AssetTypeDeduceError(#[from] asset_pipeline::AssetTypeDeduceError),
}

/// Indexed asset data.
#[derive(Debug, Clone)]
pub struct AssetData {
    pub id: Uuid,
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub metadata_content: String,
}

impl PartialEq for AssetData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AssetData {}

impl Hash for AssetData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct AssetDatabase {
    base_path: PathBuf,
    assets: HashMap<Uuid, Arc<AssetData>>,
    asset_paths: HashMap<PathBuf, Arc<AssetData>>,
}

impl AssetDatabase {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            assets: HashMap::new(),
            asset_paths: HashMap::new(),
        }
    }

    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    pub fn find_asset_by_id(&self, id: Uuid) -> Option<Arc<AssetData>> {
        self.assets.get(&id).cloned()
    }

    pub fn find_asset_by_path(&self, path: &Path) -> Option<Arc<AssetData>> {
        self.asset_paths.get(path).cloned()
    }

    /// Registers an asset from a path. It reads the metadata file; in other words, it performs IO oprations.
    pub fn register(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<Arc<AssetData>, AssetDatabaseError> {
        self.register_path(self.base_path.join(path))
    }

    fn register_path(
        &mut self,
        path: impl Into<PathBuf>,
    ) -> Result<Arc<AssetData>, AssetDatabaseError> {
        let path = path.into();
        let metadata_content = std::fs::read_to_string(path.with_extension("meta.toml"))?;
        let metadata: AssetMetadata = toml::from_str(&metadata_content)?;
        let asset_type = deduce_asset_type_from_path(&path)?;

        let asset_data = Arc::new(AssetData {
            id: metadata.id,
            path: path.clone(),
            asset_type,
            metadata_content,
        });

        self.assets.insert(metadata.id, asset_data.clone());
        self.asset_paths.insert(path, asset_data.clone());

        Ok(asset_data)
    }

    pub fn unregister(&mut self, id: &Uuid) -> Option<Arc<AssetData>> {
        match self.assets.remove(id) {
            Some(data) => {
                self.asset_paths.remove(&data.path);
                Some(data)
            }
            None => None,
        }
    }

    pub fn clear(&mut self) {
        self.assets.clear();
        self.asset_paths.clear();
    }

    /// Scans the base path for assets and registers them.
    /// It first clears the database.
    pub fn scan(&mut self) -> Result<(), AssetDatabaseError> {
        let path = self.base_path.clone();

        self.clear();
        self.scan_directory(&path)?;

        Ok(())
    }

    fn scan_directory(&mut self, path: impl AsRef<Path>) -> Result<(), AssetDatabaseError> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                self.register_path(path)?;
            } else if path.is_dir() {
                self.scan_directory(path)?;
            }
        }

        Ok(())
    }
}
