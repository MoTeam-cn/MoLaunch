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

/// 轻量更新信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateInfoLite {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: String,
    pub sha256: String,
    pub size: u64,
    pub changelog: String,
}

impl UpdateInfoLite {
    pub fn from_ffi(ffi: &FFIUpdateInfoLite) -> Self {
        Self {
            current_version: unsafe {
                if ffi.current_version.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.current_version)
                        .to_string_lossy()
                        .to_string()
                }
            },
            latest_version: unsafe {
                if ffi.latest_version.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.latest_version)
                        .to_string_lossy()
                        .to_string()
                }
            },
            update_available: ffi.update_available != 0,
            download_url: unsafe {
                if ffi.download_url.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.download_url)
                        .to_string_lossy()
                        .to_string()
                }
            },
            sha256: unsafe {
                if ffi.sha256.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.sha256)
                        .to_string_lossy()
                        .to_string()
                }
            },
            size: ffi.size,
            changelog: unsafe {
                if ffi.changelog.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.changelog)
                        .to_string_lossy()
                        .to_string()
                }
            },
        }
    }
}
