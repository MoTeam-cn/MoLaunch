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
        "mc_sdk.dll"
    }
    #[cfg(target_os = "macos")]
    {
        "mc_sdk.dylib"
    }
    #[cfg(target_os = "linux")]
    {
        "mc_sdk.so"
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
    pub isolation_mode: u32,
    pub window_title: *const std::ffi::c_char,
    pub mirror_url_meta: *const std::ffi::c_char,
    pub mirror_url_download: *const std::ffi::c_char,
    pub mirror_mode: u32,
    pub max_download_speed: u64,
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

/// Java 运行时结构体
#[repr(C)]
#[derive(Debug)]
pub struct FFIJavaRuntime {
    pub executable: *mut std::ffi::c_char,
    pub version: *mut std::ffi::c_char,
    pub major_version: u32,
    pub arch: *mut std::ffi::c_char,
    pub home: *mut std::ffi::c_char,
}

/// Java 列表结构体
#[repr(C)]
#[derive(Debug)]
pub struct FFIJavaList {
    pub runtimes: *mut FFIJavaRuntime,
    pub count: u32,
    pub error_code: i32,
    pub error_message: *mut std::ffi::c_char,
}

/// 系统内存信息结构体
#[repr(C)]
#[derive(Debug)]
pub struct FFISystemMemory {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f64,
}

/// 下载进度快照结构体
#[repr(C)]
#[derive(Debug)]
pub struct FFIProgressSnapshot {
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

/// 合并安装请求结构体
#[repr(C)]
#[derive(Debug)]
pub struct FFIMergedInstallRequest {
    pub mc_version: *const std::ffi::c_char,
    pub forge_version: *const std::ffi::c_char,
    pub neoforge_version: *const std::ffi::c_char,
    pub fabric_version: *const std::ffi::c_char,
    pub optifine_version: *const std::ffi::c_char,
    pub liteloader_version: *const std::ffi::c_char,
    pub instance_name: *const std::ffi::c_char,
}

/// 下载进度回调函数类型 (新签名)
pub type DownloadCallback = unsafe extern "C" fn(
    *const std::ffi::c_char,  // stage string
    usize,                     // current
    usize,                     // total
    u64,                       // bytes_downloaded
    u64,                       // bytes_total
    u64,                       // speed
    usize,                     // files_remaining
    *mut std::ffi::c_void,     // user_data
);

// FFI 函数类型定义
type McSdkInit = unsafe extern "C" fn(*const MCConfig) -> *mut std::ffi::c_void;
type McSdkFree = unsafe extern "C" fn(*mut std::ffi::c_void);
type McSdkVersion = unsafe extern "C" fn() -> *const std::ffi::c_char;
type McSdkLastError = unsafe extern "C" fn() -> *const ErrorInfo;
type McSdkFreeString = unsafe extern "C" fn(*mut std::ffi::c_char);
type McGetDeviceId = unsafe extern "C" fn() -> *mut std::ffi::c_char;
type McAuthOffline = unsafe extern "C" fn(*const std::ffi::c_char, *mut FFIAuthResult) -> i32;
type McAuthFreeResult = unsafe extern "C" fn(*mut FFIAuthResult);
type McListVersions = unsafe extern "C" fn(*const std::ffi::c_void, *mut FFIVersionList) -> i32;
type McFreeVersionList = unsafe extern "C" fn(*mut FFIVersionList);
/// mc_download_version(SDKHandle*, const char*, FFICallback, void*) -> i32
type McDownloadVersion = unsafe extern "C" fn(
    *const std::ffi::c_void,
    *const std::ffi::c_char,
    DownloadCallback,
    *mut std::ffi::c_void,
) -> i32;
type McDetectJava = unsafe extern "C" fn(*mut FFIJavaRuntime) -> i32;
type McListJava = unsafe extern "C" fn(*mut FFIJavaList) -> i32;
type McFreeJavaRuntime = unsafe extern "C" fn(*mut FFIJavaRuntime);
type McFreeJavaList = unsafe extern "C" fn(*mut FFIJavaList);
type McListInstalledVersions =
    unsafe extern "C" fn(*const std::ffi::c_void, *mut *mut *mut std::ffi::c_char, *mut u32) -> i32;
type McFreeStringArray = unsafe extern "C" fn(*mut *mut std::ffi::c_char, u32);

type McGetSystemMemory = unsafe extern "C" fn(*mut FFISystemMemory) -> i32;
type McGetProgress = unsafe extern "C" fn(*mut FFIProgressSnapshot) -> i32;
type McResetProgress = unsafe extern "C" fn() -> i32;
type McIsDownloading = unsafe extern "C" fn() -> i32;
type McSetWindowTitle = unsafe extern "C" fn(u32, *const std::ffi::c_char) -> i32;
type McStopWindowTitle = unsafe extern "C" fn() -> i32;
type McLaunchGameEx = unsafe extern "C" fn(
    *const std::ffi::c_void,  // handle
    *const std::ffi::c_char,  // username
    *const std::ffi::c_char,  // uuid
    *const std::ffi::c_char,  // access_token
    *const std::ffi::c_char,  // version_id
    u32,                       // min_memory
    u32,                       // max_memory
    u32,                       // window_width
    u32,                       // window_height
    *const std::ffi::c_char,  // server_address
    u32,                       // server_port
) -> i32;
type McListForgeVersions = unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_char, *mut *mut std::ffi::c_char) -> i32;
type McListNeoforgeVersions = unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_char, *mut *mut std::ffi::c_char) -> i32;
type McListFabricVersions = unsafe extern "C" fn(*const std::ffi::c_void, *mut *mut std::ffi::c_char) -> i32;
type McListOptifineVersions = unsafe extern "C" fn(*const std::ffi::c_void, *mut *mut std::ffi::c_char) -> i32;
type McListLiteloaderVersions = unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_char, *mut *mut std::ffi::c_char) -> i32;
type McValidateLoaders = unsafe extern "C" fn(*const std::ffi::c_char, *const std::ffi::c_char, *const std::ffi::c_char, *const std::ffi::c_char, *const std::ffi::c_char) -> i32;
type McInstallMerged = unsafe extern "C" fn(*const std::ffi::c_void, *const FFIMergedInstallRequest, *const std::ffi::c_void, *mut std::ffi::c_void) -> i32;

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
    pub download_version: McDownloadVersion,
    pub detect_java: McDetectJava,
    pub list_java: McListJava,
    pub free_java_runtime: McFreeJavaRuntime,
    pub free_java_list: McFreeJavaList,
    pub list_installed_versions: McListInstalledVersions,
    pub free_string_array: McFreeStringArray,
    pub get_system_memory: McGetSystemMemory,
    pub get_progress: McGetProgress,
    pub reset_progress: McResetProgress,
    pub is_downloading: McIsDownloading,
    pub set_window_title: McSetWindowTitle,
    pub stop_window_title: McStopWindowTitle,
    pub launch_game_ex: McLaunchGameEx,
    pub list_forge_versions: McListForgeVersions,
    pub list_neoforge_versions: McListNeoforgeVersions,
    pub list_fabric_versions: McListFabricVersions,
    pub list_optifine_versions: McListOptifineVersions,
    pub list_liteloader_versions: McListLiteloaderVersions,
    pub validate_loaders: McValidateLoaders,
    pub install_merged: McInstallMerged,
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
            libloading::Library::new(&lib_path)
                .map_err(|e| SdkError::LoadFailed(format!("Failed to load library: {}", e)))?
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
                download_version: *lib.get(b"mc_download_version").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_download_version: {}", e))
                })?,
                detect_java: *lib.get(b"mc_detect_java").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_detect_java: {}", e))
                })?,
                list_java: *lib.get(b"mc_list_java").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_list_java: {}", e))
                })?,
                free_java_runtime: *lib.get(b"mc_free_java_runtime").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_free_java_runtime: {}", e))
                })?,
                free_java_list: *lib.get(b"mc_free_java_list").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_free_java_list: {}", e))
                })?,
                list_installed_versions: *lib.get(b"mc_list_installed_versions").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_list_installed_versions: {}", e))
                })?,
                free_string_array: *lib.get(b"mc_free_string_array").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_free_string_array: {}", e))
                })?,
                get_system_memory: *lib.get(b"mc_get_system_memory").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_get_system_memory: {}", e))
                })?,
                get_progress: *lib.get(b"mc_get_progress").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_get_progress: {}", e))
                })?,
                reset_progress: *lib.get(b"mc_reset_progress").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_reset_progress: {}", e))
                })?,
                is_downloading: *lib.get(b"mc_is_downloading").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_is_downloading: {}", e))
                })?,
                set_window_title: *lib.get(b"mc_set_window_title").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_set_window_title: {}", e))
                })?,
                stop_window_title: *lib.get(b"mc_stop_window_title").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_stop_window_title: {}", e))
                })?,
                launch_game_ex: *lib.get(b"mc_launch_game_ex").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_launch_game_ex: {}", e))
                })?,
                list_forge_versions: *lib.get(b"mc_list_forge_versions").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_list_forge_versions: {}", e))
                })?,
                list_neoforge_versions: *lib.get(b"mc_list_neoforge_versions").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_list_neoforge_versions: {}", e))
                })?,
                list_fabric_versions: *lib.get(b"mc_list_fabric_versions").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_list_fabric_versions: {}", e))
                })?,
                list_optifine_versions: *lib.get(b"mc_list_optifine_versions").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_list_optifine_versions: {}", e))
                })?,
                list_liteloader_versions: *lib.get(b"mc_list_liteloader_versions").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_list_liteloader_versions: {}", e))
                })?,
                validate_loaders: *lib.get(b"mc_validate_loaders").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_validate_loaders: {}", e))
                })?,
                install_merged: *lib.get(b"mc_install_merged").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_install_merged: {}", e))
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
    pub fn init(
        &mut self,
        game_dir: &str,
        max_threads: u32,
        log_level: u32,
        mirror_url: Option<&str>,
        mirror_url_meta: Option<&str>,
        mirror_url_download: Option<&str>,
        mirror_mode: u32,
        max_download_speed: u64,
    ) -> Result<(), SdkError> {
        let game_dir_cstr = std::ffi::CString::new(game_dir)
            .map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
        let mirror_cstr = match mirror_url {
            Some(url) if !url.is_empty() => Some(std::ffi::CString::new(url)
                .map_err(|e| SdkError::InvalidParameter(e.to_string()))?),
            _ => None,
        };
        let mirror_meta_cstr = match mirror_url_meta {
            Some(url) if !url.is_empty() => Some(std::ffi::CString::new(url)
                .map_err(|e| SdkError::InvalidParameter(e.to_string()))?),
            _ => None,
        };
        let mirror_download_cstr = match mirror_url_download {
            Some(url) if !url.is_empty() => Some(std::ffi::CString::new(url)
                .map_err(|e| SdkError::InvalidParameter(e.to_string()))?),
            _ => None,
        };

        let config = MCConfig {
            game_dir: game_dir_cstr.as_ptr(),
            max_download_threads: max_threads,
            mirror_url: mirror_cstr.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            log_level,
            curseforge_api_key: std::ptr::null(),
            isolation_mode: 0,
            window_title: std::ptr::null(),
            mirror_url_meta: mirror_meta_cstr.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            mirror_url_download: mirror_download_cstr.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            mirror_mode,
            max_download_speed,
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

    /// 获取 SDK handle 原始指针（用于跨线程 FFI 调用）
    pub fn handle_ptr(&self) -> *const std::ffi::c_void {
        self.handle
    }

    /// 获取 mc_download_version 函数指针地址（usize，可跨线程传递）
    pub fn download_fn_addr(&self) -> usize {
        self.functions.download_version as usize
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
        let mut version_list = FFIVersionList {
            versions: std::ptr::null_mut(),
            count: 0,
            latest_release: std::ptr::null_mut(),
            latest_snapshot: std::ptr::null_mut(),
            error_code: 0,
            error_message: std::ptr::null_mut(),
        };

        // v0.1.8: mc_list_versions 需要传入 handle，传 NULL 走官方源
        let code = unsafe { (self.functions.list_versions)(self.handle, &mut version_list) };

        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }

        let result = VersionList::from_ffi(&version_list);

        // 释放 FFI 内存
        unsafe { (self.functions.free_version_list)(&mut version_list) };

        Ok(result)
    }

    /// 下载版本（带进度回调）
    pub fn download_version_with_callback<F>(
        &self,
        version_id: &str,
        callback: F,
    ) -> Result<(), SdkError>
    where
        F: Fn(&str, usize, usize, u64, u64, u64, usize) + Send + 'static,
    {
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }

        let version_cstr = std::ffi::CString::new(version_id)
            .map_err(|e| SdkError::InvalidParameter(e.to_string()))?;

        // 将闭包包装为 Box 以便传递给 C
        let callback_box = Box::new(callback);
        let callback_ptr = Box::into_raw(callback_box) as *mut std::ffi::c_void;

        // 定义 C 回调函数，签名与 C 端新 FFICallback 一致
        unsafe extern "C" fn c_callback(
            stage: *const std::ffi::c_char,
            current: usize,
            total: usize,
            bytes_downloaded: u64,
            bytes_total: u64,
            speed: u64,
            files_remaining: usize,
            user_data: *mut std::ffi::c_void,
        ) {
            if !user_data.is_null() && !stage.is_null() {
                let callback = &*(user_data
                    as *const Box<dyn Fn(&str, usize, usize, u64, u64, u64, usize) + Send>);
                let stage_str = std::ffi::CStr::from_ptr(stage)
                    .to_string_lossy()
                    .to_string();
                callback(
                    &stage_str,
                    current,
                    total,
                    bytes_downloaded,
                    bytes_total,
                    speed,
                    files_remaining,
                );
            }
        }

        let code = unsafe {
            (self.functions.download_version)(
                self.handle,
                version_cstr.as_ptr(),
                c_callback,
                callback_ptr,
            )
        };

        // 释放回调内存
        unsafe {
            let _ = Box::from_raw(
                callback_ptr
                    as *mut Box<dyn Fn(&str, usize, usize, u64, u64, u64, usize) + Send>,
            );
        }

        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }

        log::info!("Version {} downloaded successfully", version_id);
        Ok(())
    }

    /// 下载版本（无进度回调）
    pub fn download_version(&self, version_id: &str) -> Result<(), SdkError> {
        self.download_version_with_callback(version_id, |_, _, _, _, _, _, _| {})
    }

    /// 检测 Java
    pub fn detect_java(&self) -> Result<JavaRuntime, SdkError> {
        let mut java = FFIJavaRuntime {
            executable: std::ptr::null_mut(),
            version: std::ptr::null_mut(),
            major_version: 0,
            arch: std::ptr::null_mut(),
            home: std::ptr::null_mut(),
        };

        let code = unsafe { (self.functions.detect_java)(&mut java) };

        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }

        let result = JavaRuntime::from_ffi(&java);
        unsafe { (self.functions.free_java_runtime)(&mut java) };

        Ok(result)
    }

    /// 列出所有 Java
    pub fn list_java(&self) -> Result<Vec<JavaRuntime>, SdkError> {
        let mut java_list = FFIJavaList {
            runtimes: std::ptr::null_mut(),
            count: 0,
            error_code: 0,
            error_message: std::ptr::null_mut(),
        };

        let code = unsafe { (self.functions.list_java)(&mut java_list) };

        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }

        let mut result = Vec::new();
        if !java_list.runtimes.is_null() && java_list.count > 0 {
            for i in 0..java_list.count {
                let entry = unsafe { &*java_list.runtimes.add(i as usize) };
                result.push(JavaRuntime::from_ffi(entry));
            }
        }

        unsafe { (self.functions.free_java_list)(&mut java_list) };

        Ok(result)
    }

    /// 获取已安装版本列表
    pub fn list_installed_versions(&self) -> Result<Vec<String>, SdkError> {
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }

        let mut versions: *mut *mut std::ffi::c_char = std::ptr::null_mut();
        let mut count: u32 = 0;

        let code = unsafe {
            (self.functions.list_installed_versions)(self.handle, &mut versions, &mut count)
        };

        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }

        let mut result = Vec::new();
        if !versions.is_null() && count > 0 {
            for i in 0..count {
                let ptr = unsafe { *versions.add(i as usize) };
                if !ptr.is_null() {
                    let s = unsafe { std::ffi::CStr::from_ptr(ptr) }
                        .to_string_lossy()
                        .to_string();
                    result.push(s);
                }
            }
        }

        unsafe { (self.functions.free_string_array)(versions, count) };

        Ok(result)
    }

    /// 获取系统内存信息
    pub fn get_system_memory(&self) -> Result<SystemMemory, SdkError> {
        let mut mem = FFISystemMemory {
            total: 0,
            used: 0,
            available: 0,
            usage_percent: 0.0,
        };
        let code = unsafe { (self.functions.get_system_memory)(&mut mem) };
        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }
        Ok(SystemMemory::from_ffi(&mem))
    }

    /// 获取下载进度快照
    pub fn get_progress(&self) -> Result<ProgressSnapshot, SdkError> {
        let mut snapshot = FFIProgressSnapshot {
            stage: 0,
            current: 0,
            total: 0,
            bytes_downloaded: 0,
            bytes_total: 0,
            speed: 0,
            files_remaining: 0,
            is_active: false,
            is_complete: false,
            error_code: 0,
        };

        let code = unsafe { (self.functions.get_progress)(&mut snapshot) };
        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }

        Ok(ProgressSnapshot::from_ffi(&snapshot))
    }

    /// 重置下载进度
    pub fn reset_progress(&self) -> Result<(), SdkError> {
        let code = unsafe { (self.functions.reset_progress)() };
        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }
        Ok(())
    }

    /// 检查是否正在下载
    pub fn is_downloading(&self) -> bool {
        unsafe { (self.functions.is_downloading)() == 1 }
    }

    /// 启动游戏（扩展版本）
    #[allow(clippy::too_many_arguments)]
    pub fn launch_game_ex(
        &self,
        username: &str,
        uuid: &str,
        access_token: &str,
        version_id: &str,
        min_memory: u32,
        max_memory: u32,
        window_width: u32,
        window_height: u32,
        server_address: Option<&str>,
        server_port: u32,
    ) -> Result<(), SdkError> {
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }

        let username_c = std::ffi::CString::new(username)
            .map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
        let uuid_c =
            std::ffi::CString::new(uuid).map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
        let token_c = std::ffi::CString::new(access_token)
            .map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
        let version_c = std::ffi::CString::new(version_id)
            .map_err(|e| SdkError::InvalidParameter(e.to_string()))?;

        let server_c = match server_address {
            Some(addr) => Some(
                std::ffi::CString::new(addr)
                    .map_err(|e| SdkError::InvalidParameter(e.to_string()))?,
            ),
            None => None,
        };
        let server_ptr = server_c
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());

        let code = unsafe {
            (self.functions.launch_game_ex)(
                self.handle,
                username_c.as_ptr(),
                uuid_c.as_ptr(),
                token_c.as_ptr(),
                version_c.as_ptr(),
                min_memory,
                max_memory,
                window_width,
                window_height,
                server_ptr,
                server_port,
            )
        };

        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }

        Ok(())
    }

    /// 获取 mc_install_merged 函数指针地址
    pub fn install_merged_fn_addr(&self) -> usize {
        self.functions.install_merged as usize
    }

    /// 查询 Forge 版本列表
    pub fn list_forge_versions(&self, mc_version: &str) -> Result<String, SdkError> {
        if self.handle.is_null() { return Err(SdkError::NotInitialized); }
        let mc_cstr = std::ffi::CString::new(mc_version).map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
        let mut result: *mut std::ffi::c_char = std::ptr::null_mut();
        let code = unsafe { (self.functions.list_forge_versions)(self.handle, mc_cstr.as_ptr(), &mut result) };
        if code != 0 { return Err(SdkError::FfiFailed(code)); }
        let json = if result.is_null() {
            String::from("[]")
        } else {
            let s = unsafe { std::ffi::CStr::from_ptr(result) }.to_string_lossy().to_string();
            unsafe { (self.functions.free_string)(result) };
            if s.is_empty() || s == "null" { String::from("[]") } else { s }
        };
        log::info!("Forge versions for {}: {} items", mc_version, json.matches('"').count() / 2);
        Ok(json)
    }

    /// 查询 NeoForge 版本列表
    pub fn list_neoforge_versions(&self, mc_version: &str) -> Result<String, SdkError> {
        if self.handle.is_null() { return Err(SdkError::NotInitialized); }
        let mc_cstr = std::ffi::CString::new(mc_version).map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
        let mut result: *mut std::ffi::c_char = std::ptr::null_mut();
        let code = unsafe { (self.functions.list_neoforge_versions)(self.handle, mc_cstr.as_ptr(), &mut result) };
        if code != 0 { return Err(SdkError::FfiFailed(code)); }
        let json = if result.is_null() {
            String::from("[]")
        } else {
            let s = unsafe { std::ffi::CStr::from_ptr(result) }.to_string_lossy().to_string();
            unsafe { (self.functions.free_string)(result) };
            if s.is_empty() || s == "null" { String::from("[]") } else { s }
        };
        Ok(json)
    }

    /// 查询 Fabric 版本列表
    pub fn list_fabric_versions(&self) -> Result<String, SdkError> {
        if self.handle.is_null() { return Err(SdkError::NotInitialized); }
        let mut result: *mut std::ffi::c_char = std::ptr::null_mut();
        let code = unsafe { (self.functions.list_fabric_versions)(self.handle, &mut result) };
        if code != 0 { return Err(SdkError::FfiFailed(code)); }
        let json = if result.is_null() {
            String::from("[]")
        } else {
            let s = unsafe { std::ffi::CStr::from_ptr(result) }.to_string_lossy().to_string();
            unsafe { (self.functions.free_string)(result) };
            if s.is_empty() || s == "null" { String::from("[]") } else { s }
        };
        log::info!("Fabric versions: {}", &json[..json.len().min(100)]);
        Ok(json)
    }

    /// 查询 OptiFine 版本列表
    pub fn list_optifine_versions(&self) -> Result<String, SdkError> {
        if self.handle.is_null() { return Err(SdkError::NotInitialized); }
        let mut result: *mut std::ffi::c_char = std::ptr::null_mut();
        let code = unsafe { (self.functions.list_optifine_versions)(self.handle, &mut result) };
        if code != 0 { return Err(SdkError::FfiFailed(code)); }
        let json = if result.is_null() {
            String::from("[]")
        } else {
            let s = unsafe { std::ffi::CStr::from_ptr(result) }.to_string_lossy().to_string();
            unsafe { (self.functions.free_string)(result) };
            if s.is_empty() || s == "null" { String::from("[]") } else { s }
        };
        Ok(json)
    }

    /// 查询 LiteLoader 版本列表
    pub fn list_liteloader_versions(&self, mc_version: &str) -> Result<String, SdkError> {
        if self.handle.is_null() { return Err(SdkError::NotInitialized); }
        let mc_cstr = std::ffi::CString::new(mc_version).map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
        let mut result: *mut std::ffi::c_char = std::ptr::null_mut();
        let code = unsafe { (self.functions.list_liteloader_versions)(self.handle, mc_cstr.as_ptr(), &mut result) };
        if code != 0 { return Err(SdkError::FfiFailed(code)); }
        let json = if result.is_null() {
            String::from("[]")
        } else {
            let s = unsafe { std::ffi::CStr::from_ptr(result) }.to_string_lossy().to_string();
            unsafe { (self.functions.free_string)(result) };
            if s.is_empty() || s == "null" { String::from("[]") } else { s }
        };
        Ok(json)
    }

    /// 合并安装
    #[allow(clippy::too_many_arguments)]
    pub fn install_merged(
        &self,
        mc_version: &str,
        forge_version: Option<&str>,
        neoforge_version: Option<&str>,
        fabric_version: Option<&str>,
        optifine_version: Option<&str>,
        liteloader_version: Option<&str>,
        instance_name: Option<&str>,
    ) -> Result<(), SdkError> {
        if self.handle.is_null() { return Err(SdkError::NotInitialized); }

        let mc_c = std::ffi::CString::new(mc_version).map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
        let forge_c = forge_version.map(|v| std::ffi::CString::new(v).unwrap());
        let neoforge_c = neoforge_version.map(|v| std::ffi::CString::new(v).unwrap());
        let fabric_c = fabric_version.map(|v| std::ffi::CString::new(v).unwrap());
        let optifine_c = optifine_version.map(|v| std::ffi::CString::new(v).unwrap());
        let liteloader_c = liteloader_version.map(|v| std::ffi::CString::new(v).unwrap());
        let instance_c = instance_name.map(|v| std::ffi::CString::new(v).unwrap());

        let request = FFIMergedInstallRequest {
            mc_version: mc_c.as_ptr(),
            forge_version: forge_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            neoforge_version: neoforge_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            fabric_version: fabric_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            optifine_version: optifine_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            liteloader_version: liteloader_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            instance_name: instance_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
        };

        let code = unsafe { (self.functions.install_merged)(self.handle, &request, std::ptr::null(), std::ptr::null_mut()) };
        if code != 0 { return Err(SdkError::FfiFailed(code)); }
        Ok(())
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
    fn from_ffi(ffi: &FFIJavaRuntime) -> Self {
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

/// 系统内存信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemMemory {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f64,
}

impl SystemMemory {
    fn from_ffi(ffi: &FFISystemMemory) -> Self {
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
    fn from_ffi(ffi: &FFIProgressSnapshot) -> Self {
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

/// 获取系统内存信息（独立函数，无需 SDK 句柄）
pub fn get_system_memory_static() -> Result<SystemMemory, SdkError> {
    let lib_path = check_sdk_library()?;
    let lib = unsafe {
        libloading::Library::new(&lib_path)
            .map_err(|e| SdkError::LoadFailed(format!("Failed to load library: {}", e)))?
    };

    let func: McGetSystemMemory = unsafe {
        *lib.get(b"mc_get_system_memory").map_err(|e| {
            SdkError::LoadFailed(format!("Failed to get mc_get_system_memory: {}", e))
        })?
    };

    let mut memory = FFISystemMemory {
        total: 0,
        used: 0,
        available: 0,
        usage_percent: 0.0,
    };

    let code = unsafe { func(&mut memory) };
    if code != 0 {
        return Err(SdkError::FfiFailed(code));
    }

    Ok(SystemMemory::from_ffi(&memory))
}

/// 校验加载器兼容性（独立函数，无需 SDK 句柄）
pub fn validate_loaders(
    mc_version: &str,
    forge_version: Option<&str>,
    neoforge_version: Option<&str>,
    fabric_version: Option<&str>,
    optifine_version: Option<&str>,
) -> Result<(), SdkError> {
    let lib_path = check_sdk_library()?;
    let lib = unsafe {
        libloading::Library::new(&lib_path)
            .map_err(|e| SdkError::LoadFailed(format!("Failed to load library: {}", e)))?
    };
    let func: McValidateLoaders = unsafe {
        *lib.get(b"mc_validate_loaders").map_err(|e| {
            SdkError::LoadFailed(format!("Failed to get mc_validate_loaders: {}", e))
        })?
    };

    let mc_c = std::ffi::CString::new(mc_version).map_err(|e| SdkError::InvalidParameter(e.to_string()))?;
    let forge_c = forge_version.map(|v| std::ffi::CString::new(v).unwrap());
    let neoforge_c = neoforge_version.map(|v| std::ffi::CString::new(v).unwrap());
    let fabric_c = fabric_version.map(|v| std::ffi::CString::new(v).unwrap());
    let optifine_c = optifine_version.map(|v| std::ffi::CString::new(v).unwrap());

    let code = unsafe {
        func(
            mc_c.as_ptr(),
            forge_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            neoforge_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            fabric_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            optifine_c.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
        )
    };

    if code != 0 {
        return Err(SdkError::FfiFailed(code));
    }
    Ok(())
}
