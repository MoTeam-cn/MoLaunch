//! 加载器基础类型

use serde::{Deserialize, Serialize};

/// Loader type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoaderType {
    Forge,
    NeoForge,
    Fabric,
    OptiFine,
    LiteLoader,
}

/// Loader version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    pub version: String,
    pub is_recommended: bool,
    pub release_time: Option<String>,
}