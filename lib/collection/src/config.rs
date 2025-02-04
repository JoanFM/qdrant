use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use atomicwrites::AtomicFile;
use atomicwrites::OverwriteBehavior::AllowOverwrite;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use wal::WalOptions;

use segment::types::{Distance, HnswConfig};

use crate::collection_builder::optimizers_builder::OptimizersConfig;
use crate::operations::types::{CollectionError, CollectionResult};

pub const COLLECTION_CONFIG_FILE: &str = "config.json";

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct WalConfig {
    /// Size of a single WAL segment in MB
    pub wal_capacity_mb: usize,
    /// Number of WAL segments to create ahead of actually used ones
    pub wal_segments_ahead: usize,
}

impl From<&WalConfig> for WalOptions {
    fn from(config: &WalConfig) -> Self {
        WalOptions {
            segment_capacity: config.wal_capacity_mb * 1024 * 1024,
            segment_queue_len: config.wal_segments_ahead
        }
    }
}

impl Default for WalConfig {
    fn default() -> Self { WalConfig { wal_capacity_mb: 32, wal_segments_ahead: 0 } }
}


#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub struct CollectionParams {
    /// Size of a vectors used
    pub vector_size: usize,
    /// Type of distance function used for measuring distance between vectors
    pub distance: Distance
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct CollectionConfig {
    pub params: CollectionParams,
    pub hnsw_config: HnswConfig,
    pub optimizer_config: OptimizersConfig,
    pub wal_config: WalConfig,
}


impl CollectionConfig {
    pub fn save(&self, path: &Path) -> CollectionResult<()> {
        let config_path = path.join(COLLECTION_CONFIG_FILE);
        let af = AtomicFile::new(&config_path, AllowOverwrite);
        let state_bytes = serde_json::to_vec(self).unwrap();
        af.write(|f| {
            f.write_all(&state_bytes)
        }).or_else(move |err|
            Err(CollectionError::ServiceError {
                error: format!("Can't write {:?}, error: {}", config_path, err)
            })
        )?;
        Ok(())
    }

    pub fn load(path: &Path) -> CollectionResult<Self> {
        let config_path = path.join(COLLECTION_CONFIG_FILE);
        let mut contents = String::new();
        let mut file = File::open(config_path)?;
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    }
}
