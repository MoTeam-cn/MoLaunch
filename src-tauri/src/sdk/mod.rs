//! McSDK FFI 绑定层
//!
//! 提供跨平台 SDK 动态库加载和 FFI 函数绑定

use std::path::PathBuf;
use thiserror::Error;

/// SDK 错误类型
#[derive(Error, Debug)]
pub enum SdkError {
    #[error("SDK not initialized")]
    NotInitialized,
    #[error("Failed to load SDK library: {0}")]
    LoadFailed(String),
    #[error("FFI call failed with code: {0}")]
    FfiFailed(i32),
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Null pointer returned")]
    NullPointer,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// 获取当前平台的 SDK 文件名
pub fn get_sdk_filename() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "mc_sdk-windows-x86_64.dll"
    }
    #[cfg(target_os = "macos")]
    {
        if cfg!(target_arch = "aarch64") {
            "mc_sdk-macos-aarch64.dylib"
        } else {
            "mc_sdk-macos-x86_64.dylib"
        }
    }
    #[cfg(target_os = "linux")]
    {
        "mc_sdk-linux-x86_64.so"
    }
}

/// 获取 SDK 资源目录路径
pub fn get_sdk_resource_dir() -> PathBuf {
    // 在开发模式下，sdk_data 在项目根目录
    // 在发布模式下，sdk_data 被打包到 resources 目录
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sdk_data");
    
    if dev_path.exists() {
        return dev_path;
    }
    
    // 发布模式下，尝试从 Tauri 资源目录获取
    if let Ok(resource_dir) = std::env::current_exe() {
        if let Some(exe_dir) = resource_dir.parent() {
            let resource_path = exe_dir.join("resources").join("sdk_data");
            if resource_path.exists() {
                return resource_path;
            }
        }
    }
    
    // 兜底返回开发路径
    dev_path
}

/// 获取 SDK 动态库的完整路径
pub fn get_sdk_library_path() -> PathBuf {
    let resource_dir = get_sdk_resource_dir();
    let filename = get_sdk_filename();
    resource_dir.join(filename)
}

/// 检查 SDK 库是否存在
pub fn check_sdk_library() -> Result<PathBuf, SdkError> {
    let path = get_sdk_library_path();
    if path.exists() {
        Ok(path)
    } else {
        Err(SdkError::LoadFailed(format!(
            "SDK library not found at: {}",
            path.display()
        )))
    }
}

/// SDK 配置结构体
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MCConfig {
    pub game_dir: *const std::ffi::c_char,
    pub max_download_threads: u32,
    pub mirror_url: *const std::ffi::c_char,
    pub log_level: u32,
    pub curseforge_api_key: *const std::ffi::c_char,
}

/// 认证结果结构体
#[repr(C)]
#[derive(Debug)]
pub struct FFIAuthResult {
    pub auth_type: i32,
    pub access_token: *mut std::ffi::c_char,
    pub refresh_token: *mut std::ffi::c_char,
    pub uuid: *mut std::ffi::c_char,
    pub username: *mut std::ffi::c_char,
    pub expires_at: i64,
    pub error_code: i32,
    pub error_message: *mut std::ffi::c_char,
}

/// 版本信息结构体
#[repr(C)]
#[derive(Debug)]
pub struct FFIVersionEntry {
    pub id: *mut std::ffi::c_char,
    pub version_type: *mut std::ffi::c_char,
    pub release_time: i64,
}

/// 版本列表结构体
#[repr(C)]
#[derive(Debug)]
pub struct FFIVersionList {
    pub versions: *mut FFIVersionEntry,
    pub count: u32,
    pub latest_release: *mut std::ffi::c_char,
    pub latest_snapshot: *mut std::ffi::c_char,
    pub error_code: i32,
    pub error_message: *mut std::ffi::c_char,
}

/// 错误信息结构体
#[repr(C)]
#[derive(Debug)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: *const std::ffi::c_char,
}

// FFI 函数类型定义
type McSdkInit = unsafe extern "C" fn(*const MCConfig) -> *mut std::ffi::c_void;
type McSdkFree = unsafe extern "C" fn(*mut std::ffi::c_void);
type McSdkVersion = unsafe extern "C" fn() -> *const std::ffi::c_char;
type McSdkLastError = unsafe extern "C" fn() -> *const ErrorInfo;
type McSdkFreeString = unsafe extern "C" fn(*mut std::ffi::c_char);
type McGetDeviceId = unsafe extern "C" fn() -> *mut std::ffi::c_char;
type McAuthOffline = unsafe extern "C" fn(*const std::ffi::c_char, *mut FFIAuthResult) -> i32;
type McAuthFreeResult = unsafe extern "C" fn(*mut FFIAuthResult);
type McListVersions = unsafe extern "C" fn(*mut FFIVersionList) -> i32;
type McFreeVersionList = unsafe extern "C" fn(*mut FFIVersionList);

/// SDK 函数集合
pub struct SdkFunctions {
    pub init: McSdkInit,
    pub free: McSdkFree,
    pub version: McSdkVersion,
    pub last_error: McSdkLastError,
    pub free_string: McSdkFreeString,
    pub get_device_id: McGetDeviceId,
    pub auth_offline: McAuthOffline,
    pub auth_free_result: McAuthFreeResult,
    pub list_versions: McListVersions,
    pub free_version_list: McFreeVersionList,
}

/// SDK 实例
pub struct SdkInstance {
    handle: *mut std::ffi::c_void,
    functions: SdkFunctions,
    _lib: libloading::Library,
}

unsafe impl Send for SdkInstance {}
unsafe impl Sync for SdkInstance {}

impl SdkInstance {
    /// 加载 SDK 库
    pub fn load() -> Result<Self, SdkError> {
        let lib_path = check_sdk_library()?;
        
        log::info!("Loading SDK from: {}", lib_path.display());
        
        let lib = unsafe {
            libloading::Library::new(&lib_path).map_err(|e| {
                SdkError::LoadFailed(format!("Failed to load library: {}", e))
            })?
        };
        
        let functions = unsafe {
            SdkFunctions {
                init: *lib.get(b"mc_sdk_init").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_sdk_init: {}", e))
                })?,
                free: *lib.get(b"mc_sdk_free").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_sdk_free: {}", e))
                })?,
                version: *lib.get(b"mc_sdk_version").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_sdk_version: {}", e))
                })?,
                last_error: *lib.get(b"mc_sdk_last_error").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_sdk_last_error: {}", e))
                })?,
                free_string: *lib.get(b"mc_sdk_free_string").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_sdk_free_string: {}", e))
                })?,
                get_device_id: *lib.get(b"mc_get_device_id").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_get_device_id: {}", e))
                })?,
                auth_offline: *lib.get(b"mc_auth_offline").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_auth_offline: {}", e))
                })?,
                auth_free_result: *lib.get(b"mc_auth_free_result").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_auth_free_result: {}", e))
                })?,
                list_versions: *lib.get(b"mc_list_versions").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_list_versions: {}", e))
                })?,
                free_version_list: *lib.get(b"mc_free_version_list").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_free_version_list: {}", e))
                })?,
            }
        };
        
        Ok(Self {
            handle: std::ptr::null_mut(),
            functions,
            _lib: lib,
        })
    }
    
    /// 初始化 SDK
    pub fn init(&mut self, game_dir: &str, max_threads: u32, log_level: u32) -> Result<(), SdkError> {
        let game_dir_cstr = std::ffi::CString::new(game_dir)
            .map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
        
        let config = MCConfig {
            game_dir: game_dir_cstr.as_ptr(),
            max_download_threads: max_threads,
            mirror_url: std::ptr::null(),
            log_level,
            curseforge_api_key: std::ptr::null(),
        };
        
        let handle = unsafe { (self.functions.init)(&config) };
        
        if handle.is_null() {
            let error = unsafe { (self.functions.last_error)() };
            if !error.is_null() {
                let error_ref = unsafe { &*error };
                if !error_ref.message.is_null() {
                    let _message = unsafe { std::ffi::CStr::from_ptr(error_ref.message) }
                        .to_string_lossy()
                        .to_string();
                    return Err(SdkError::FfiFailed(error_ref.code));
                }
            }
            return Err(SdkError::NullPointer);
        }
        
        self.handle = handle;
        log::info!("SDK initialized successfully");
        Ok(())
    }
    
    /// 获取 SDK 版本
    pub fn version(&self) -> String {
        let version_ptr = unsafe { (self.functions.version)() };
        if version_ptr.is_null() {
            return "unknown".to_string();
        }
        unsafe { std::ffi::CStr::from_ptr(version_ptr) }
            .to_string_lossy()
            .to_string()
    }
    
    /// 获取设备 ID
    pub fn get_device_id(&self) -> Result<String, SdkError> {
        let device_id_ptr = unsafe { (self.functions.get_device_id)() };
        if device_id_ptr.is_null() {
            return Err(SdkError::NullPointer);
        }
        
        let device_id = unsafe { std::ffi::CStr::from_ptr(device_id_ptr) }
            .to_string_lossy()
            .to_string();
        
        // 释放 SDK 分配的内存
        unsafe { (self.functions.free_string)(device_id_ptr) };
        
        Ok(device_id)
    }
    
    /// 离线登录
    pub fn auth_offline(&self, username: &str) -> Result<AuthResult, SdkError> {
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }
        
        let username_cstr = std::ffi::CString::new(username)
            .map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
        
        let mut result = FFIAuthResult {
            auth_type: 0,
            access_token: std::ptr::null_mut(),
            refresh_token: std::ptr::null_mut(),
            uuid: std::ptr::null_mut(),
            username: std::ptr::null_mut(),
            expires_at: 0,
            error_code: 0,
            error_message: std::ptr::null_mut(),
        };
        
        let code = unsafe { (self.functions.auth_offline)(username_cstr.as_ptr(), &mut result) };
        
        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }
        
        let auth_result = AuthResult::from_ffi(&result);
        
        // 释放 FFI 内存
        unsafe { (self.functions.auth_free_result)(&mut result) };
        
        Ok(auth_result)
    }
    
    /// 获取版本列表
    pub fn list_versions(&self) -> Result<VersionList, SdkError> {
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }
        
        let mut version_list = FFIVersionList {
            versions: std::ptr::null_mut(),
            count: 0,
            latest_release: std::ptr::null_mut(),
            latest_snapshot: std::ptr::null_mut(),
            error_code: 0,
            error_message: std::ptr::null_mut(),
        };
        
        let code = unsafe { (self.functions.list_versions)(&mut version_list) };
        
        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }
        
        let result = VersionList::from_ffi(&version_list);
        
        // 释放 FFI 内存
        unsafe { (self.functions.free_version_list)(&mut version_list) };
        
        Ok(result)
    }
}

impl Drop for SdkInstance {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                (self.functions.free)(self.handle);
            }
            self.handle = std::ptr::null_mut();
            log::info!("SDK instance dropped");
        }
    }
}

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
    fn from_ffi(ffi: &FFIAuthResult) -> Self {
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

impl VersionList {
    fn from_ffi(ffi: &FFIVersionList) -> Self {
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
