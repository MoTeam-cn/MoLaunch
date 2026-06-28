//! Rust 适配类型定义

use super::ffi_types::*;

/// 认证结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthResult {
    pub auth_type: i32,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub uuid: String,
    pub username: String,
    pub expires_at: i64,
}

impl AuthResult {
    pub(crate) fn from_ffi(ffi: &FFIAuthResult) -> Self {
        Self {
            auth_type: ffi.auth_type,
            access_token: unsafe {
                if ffi.access_token.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.access_token)
                        .to_string_lossy()
                        .to_string()
                }
            },
            refresh_token: unsafe {
                if ffi.refresh_token.is_null() {
                    None
                } else {
                    Some(
                        std::ffi::CStr::from_ptr(ffi.refresh_token)
                            .to_string_lossy()
                            .to_string(),
                    )
                }
            },
            uuid: unsafe {
                if ffi.uuid.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.uuid)
                        .to_string_lossy()
                        .to_string()
                }
            },
            username: unsafe {
                if ffi.username.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.username)
                        .to_string_lossy()
                        .to_string()
                }
            },
            expires_at: ffi.expires_at,
        }
    }
}

/// 版本信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionInfo {
    pub id: String,
    pub version_type: String,
    pub release_time: i64,
}

/// 版本列表
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionList {
    pub versions: Vec<VersionInfo>,
    pub latest_release: String,
    pub latest_snapshot: String,
}

/// Java 运行时信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JavaRuntime {
    pub executable: String,
    pub version: String,
    pub major_version: u32,
    pub arch: String,
    pub home: String,
}

impl JavaRuntime {
    pub(crate) fn from_ffi(ffi: &FFIJavaRuntime) -> Self {
        Self {
            executable: unsafe {
                if ffi.executable.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.executable)
                        .to_string_lossy()
                        .to_string()
                }
            },
            version: unsafe {
                if ffi.version.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.version)
                        .to_string_lossy()
                        .to_string()
                }
            },
            major_version: ffi.major_version,
            arch: unsafe {
                if ffi.arch.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.arch)
                        .to_string_lossy()
                        .to_string()
                }
            },
            home: unsafe {
                if ffi.home.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.home)
                        .to_string_lossy()
                        .to_string()
                }
            },
        }
    }
}

impl VersionList {
    pub(crate) fn from_ffi(ffi: &FFIVersionList) -> Self {
        let mut versions = Vec::new();

        if !ffi.versions.is_null() && ffi.count > 0 {
            for i in 0..ffi.count {
                let entry = unsafe { &*ffi.versions.add(i as usize) };
                versions.push(VersionInfo {
                    id: unsafe {
                        if entry.id.is_null() {
                            String::new()
                        } else {
                            std::ffi::CStr::from_ptr(entry.id)
                                .to_string_lossy()
                                .to_string()
                        }
                    },
                    version_type: unsafe {
                        if entry.version_type.is_null() {
                            String::new()
                        } else {
                            std::ffi::CStr::from_ptr(entry.version_type)
                                .to_string_lossy()
                                .to_string()
                        }
                    },
                    release_time: entry.release_time,
                });
            }
        }

        Self {
            versions,
            latest_release: unsafe {
                if ffi.latest_release.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.latest_release)
                        .to_string_lossy()
                        .to_string()
                }
            },
            latest_snapshot: unsafe {
                if ffi.latest_snapshot.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(ffi.latest_snapshot)
                        .to_string_lossy()
                        .to_string()
                }
            },
        }
    }
}

/// 系统内存信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemMemory {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f64,
}

impl SystemMemory {
    pub(crate) fn from_ffi(ffi: &FFISystemMemory) -> Self {
        Self {
            total: ffi.total,
            used: ffi.used,
            available: ffi.available,
            usage_percent: ffi.usage_percent,
        }
    }
}

/// 下载进度快照
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressSnapshot {
    pub stage: u32,
    pub current: usize,
    pub total: usize,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub speed: u64,
    pub files_remaining: usize,
    pub is_active: bool,
    pub is_complete: bool,
    pub error_code: i32,
}

impl ProgressSnapshot {
    pub(crate) fn from_ffi(ffi: &FFIProgressSnapshot) -> Self {
        Self {
            stage: ffi.stage,
            current: ffi.current,
            total: ffi.total,
            bytes_downloaded: ffi.bytes_downloaded,
            bytes_total: ffi.bytes_total,
            speed: ffi.speed,
            files_remaining: ffi.files_remaining,
            is_active: ffi.is_active,
            is_complete: ffi.is_complete,
            error_code: ffi.error_code,
        }
    }
}
