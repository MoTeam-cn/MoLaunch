//! FFI 结构体和类型别名定义

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
    pub max_download_speed: u64,  // ← 修正：先 u64
    pub mirror_mode: u32,         // ← 修正：后 u32
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

/// 下载进度回调函数类型
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
pub type McSdkInit = unsafe extern "C" fn(*const MCConfig) -> *mut std::ffi::c_void;
pub type McSdkFree = unsafe extern "C" fn(*mut std::ffi::c_void);
pub type McSdkVersion = unsafe extern "C" fn() -> *const std::ffi::c_char;
pub type McSdkLastError = unsafe extern "C" fn() -> *const ErrorInfo;
pub type McSdkFreeString = unsafe extern "C" fn(*mut std::ffi::c_char);
pub type McGetDeviceId = unsafe extern "C" fn() -> *mut std::ffi::c_char;
pub type McAuthOffline = unsafe extern "C" fn(*const std::ffi::c_char, *mut FFIAuthResult) -> i32;
pub type McAuthFreeResult = unsafe extern "C" fn(*mut FFIAuthResult);
pub type McListVersions = unsafe extern "C" fn(*const std::ffi::c_void, *mut FFIVersionList) -> i32;
pub type McFreeVersionList = unsafe extern "C" fn(*mut FFIVersionList);
/// mc_download_version(SDKHandle*, const char*, FFICallback, void*) -> i32
pub type McDownloadVersion = unsafe extern "C" fn(
    *const std::ffi::c_void,
    *const std::ffi::c_char,
    DownloadCallback,
    *mut std::ffi::c_void,
) -> i32;
pub type McDetectJava = unsafe extern "C" fn(*mut FFIJavaRuntime) -> i32;
pub type McListJava = unsafe extern "C" fn(*mut FFIJavaList) -> i32;
pub type McFreeJavaRuntime = unsafe extern "C" fn(*mut FFIJavaRuntime);
pub type McFreeJavaList = unsafe extern "C" fn(*mut FFIJavaList);
pub type McListInstalledVersions =
    unsafe extern "C" fn(*const std::ffi::c_void, *mut *mut *mut std::ffi::c_char, *mut u32) -> i32;
pub type McFreeStringArray = unsafe extern "C" fn(*mut *mut std::ffi::c_char, u32);

pub type McGetSystemMemory = unsafe extern "C" fn(*mut FFISystemMemory) -> i32;
pub type McGetProgress = unsafe extern "C" fn(*mut FFIProgressSnapshot) -> i32;
pub type McResetProgress = unsafe extern "C" fn() -> i32;
pub type McIsDownloading = unsafe extern "C" fn() -> i32;
pub type McSetWindowTitle = unsafe extern "C" fn(u32, *const std::ffi::c_char) -> i32;
pub type McStopWindowTitle = unsafe extern "C" fn() -> i32;
pub type McLaunchGameEx = unsafe extern "C" fn(
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
pub type McListForgeVersions = unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_char, *mut *mut std::ffi::c_char) -> i32;
pub type McListNeoforgeVersions = unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_char, *mut *mut std::ffi::c_char) -> i32;
pub type McListFabricVersions = unsafe extern "C" fn(*const std::ffi::c_void, *mut *mut std::ffi::c_char) -> i32;
pub type McListOptifineVersions = unsafe extern "C" fn(*const std::ffi::c_void, *mut *mut std::ffi::c_char) -> i32;
pub type McListLiteloaderVersions = unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_char, *mut *mut std::ffi::c_char) -> i32;
pub type McValidateLoaders = unsafe extern "C" fn(*const std::ffi::c_char, *const std::ffi::c_char, *const std::ffi::c_char, *const std::ffi::c_char, *const std::ffi::c_char) -> i32;
pub type McInstallMerged = unsafe extern "C" fn(*const std::ffi::c_void, *const FFIMergedInstallRequest, *const std::ffi::c_void, *mut std::ffi::c_void) -> i32;
