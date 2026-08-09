//! FFI 类型定义（lite 版本）

/// 系统内存信息结构体
#[repr(C)]
#[derive(Debug)]
pub struct FFISystemMemory {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f64,
}

/// 错误信息结构体
#[repr(C)]
#[derive(Debug)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: *const std::ffi::c_char,
    pub details: *const std::ffi::c_char,
}

/// 轻量更新信息结构体
#[repr(C)]
#[derive(Debug)]
pub struct FFIUpdateInfoLite {
    pub current_version: *mut std::ffi::c_char,
    pub latest_version: *mut std::ffi::c_char,
    pub update_available: i32,
    pub download_url: *mut std::ffi::c_char,
    pub sha256: *mut std::ffi::c_char,
    pub size: u64,
    pub changelog: *mut std::ffi::c_char,
}

// FFI 函数类型定义（lite 版本）
pub type McSdkVersion = unsafe extern "C" fn() -> *const std::ffi::c_char;
pub type McSdkLastError = unsafe extern "C" fn() -> *const ErrorInfo;
pub type McSdkClearError = unsafe extern "C" fn();
pub type McSdkFreeError = unsafe extern "C" fn(*mut ErrorInfo);
pub type McSdkFreeString = unsafe extern "C" fn(*mut std::ffi::c_char);
pub type McGetDeviceId = unsafe extern "C" fn() -> *mut std::ffi::c_char;
pub type McGetSystemMemory = unsafe extern "C" fn(*mut FFISystemMemory) -> i32;
pub type McEncryptToken = unsafe extern "C" fn(*const std::ffi::c_char) -> *mut std::ffi::c_char;
pub type McDecryptToken = unsafe extern "C" fn(*const std::ffi::c_char) -> *mut std::ffi::c_char;
pub type McDecryptTokenEx =
    unsafe extern "C" fn(*const std::ffi::c_char, *mut std::ffi::c_int) -> *mut std::ffi::c_char;
pub type McUpdateCheckLite = unsafe extern "C" fn(*mut FFIUpdateInfoLite) -> i32;
pub type McUpdateFreeInfoLite = unsafe extern "C" fn(*mut FFIUpdateInfoLite);
