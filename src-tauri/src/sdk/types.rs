//! Rust 适配类型定义（lite 版本）

use super::ffi_types::*;

/// 系统内存信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemMemory {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f64,
}

impl SystemMemory {
    pub fn from_ffi(ffi: &FFISystemMemory) -> Self {
        Self {
            total: ffi.total,
            used: ffi.used,
            available: ffi.available,
            usage_percent: ffi.usage_percent,
        }
    }
}
