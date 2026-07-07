//! SDK lite 版本绑定
//! 只绑定 11 个基础函数

use super::ffi_types::*;
use super::types::*;
use super::{check_sdk_library, SdkError};
use crate::log_info;

/// SDK lite 函数集合
pub struct SdkFunctions {
    pub version: McSdkVersion,
    pub last_error: McSdkLastError,
    pub clear_error: McSdkClearError,
    pub free_error: McSdkFreeError,
    pub free_string: McSdkFreeString,
    pub get_device_id: McGetDeviceId,
    pub get_system_memory: McGetSystemMemory,
    pub encrypt_token: McEncryptToken,
    pub decrypt_token: McDecryptToken,
    pub update_check_lite: McUpdateCheckLite,
    pub update_free_info_lite: McUpdateFreeInfoLite,
}

/// SDK 实例（lite 版本不需要 handle）
pub struct SdkInstance {
    functions: SdkFunctions,
    _lib: libloading::Library,
}

// SAFETY: `libloading::Library` 本身是 Send+Sync；`SdkFunctions` 仅持有从库导出的
// 函数指针（Plain Old Data），调用时通过 FFI 传入/传出参数，无内部可变状态、
// 无线程本地存储依赖。SDK 库本身在 C层面保证 mc_* 函数线程安全（仅读设备ID、
// 内存信息等无状态查询）。因此 SdkInstance 可安全跨线程共享。
unsafe impl Send for SdkInstance {}
unsafe impl Sync for SdkInstance {}

impl SdkInstance {
    /// 加载 SDK 库
    pub fn load() -> Result<Self, SdkError> {
        let lib_path = check_sdk_library()?;
        log_info!("Loading SDK lite from: {}", lib_path.display());

        let lib = unsafe {
            libloading::Library::new(&lib_path)
                .map_err(|e| SdkError::LoadFailed(format!("Failed to load library: {}", e)))?
        };

        let functions = unsafe {
            SdkFunctions {
                version: *lib.get(b"mc_sdk_version").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_sdk_version: {}", e))
                })?,
                last_error: *lib.get(b"mc_sdk_last_error").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_sdk_last_error: {}", e))
                })?,
                clear_error: *lib.get(b"mc_sdk_clear_error").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_sdk_clear_error: {}", e))
                })?,
                free_error: *lib.get(b"mc_sdk_free_error").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_sdk_free_error: {}", e))
                })?,
                free_string: *lib.get(b"mc_sdk_free_string").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_sdk_free_string: {}", e))
                })?,
                get_device_id: *lib.get(b"mc_get_device_id").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_get_device_id: {}", e))
                })?,
                get_system_memory: *lib.get(b"mc_get_system_memory").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_get_system_memory: {}", e))
                })?,
                encrypt_token: *lib.get(b"mc_encrypt_token").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_encrypt_token: {}", e))
                })?,
                decrypt_token: *lib.get(b"mc_decrypt_token").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_decrypt_token: {}", e))
                })?,
                update_check_lite: *lib.get(b"mc_update_check_lite").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_update_check_lite: {}", e))
                })?,
                update_free_info_lite: *lib.get(b"mc_update_free_info_lite").map_err(|e| {
                    SdkError::LoadFailed(format!("Failed to get mc_update_free_info_lite: {}", e))
                })?,
            }
        };

        Ok(Self {
            functions,
            _lib: lib,
        })
    }

    /// 获取 SDK 版本（静态内存，无需释放）
    pub fn version(&self) -> Result<String, SdkError> {
        let version_ptr = unsafe { (self.functions.version)() };
        if version_ptr.is_null() {
            return Ok("unknown".to_string());
        }
        Ok(unsafe { std::ffi::CStr::from_ptr(version_ptr) }
            .to_string_lossy()
            .to_string())
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

    /// 获取系统内存信息（无需 handle）
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

    /// 加密 Token
    pub fn encrypt_token(&self, data: &str) -> Result<String, SdkError> {
        let data_cstr =
            std::ffi::CString::new(data).map_err(|e| SdkError::InvalidParameter(e.to_string()))?;

        let result = unsafe { (self.functions.encrypt_token)(data_cstr.as_ptr()) };
        if result.is_null() {
            return Err(SdkError::NullPointer);
        }

        let encrypted = unsafe { std::ffi::CStr::from_ptr(result) }
            .to_string_lossy()
            .to_string();

        unsafe { (self.functions.free_string)(result) };

        Ok(encrypted)
    }

    /// 解密 Token
    pub fn decrypt_token(&self, encrypted: &str) -> Result<String, SdkError> {
        let encrypted_cstr = std::ffi::CString::new(encrypted)
            .map_err(|e| SdkError::InvalidParameter(e.to_string()))?;

        let result = unsafe { (self.functions.decrypt_token)(encrypted_cstr.as_ptr()) };
        if result.is_null() {
            return Err(SdkError::NullPointer);
        }

        let decrypted = unsafe { std::ffi::CStr::from_ptr(result) }
            .to_string_lossy()
            .to_string();

        unsafe { (self.functions.free_string)(result) };

        Ok(decrypted)
    }

    /// 检查更新（轻量版，无需 handle）
    pub fn update_check_lite(&self) -> Result<UpdateInfoLite, SdkError> {
        let mut info = FFIUpdateInfoLite {
            current_version: std::ptr::null_mut(),
            latest_version: std::ptr::null_mut(),
            update_available: 0,
            download_url: std::ptr::null_mut(),
            sha256: std::ptr::null_mut(),
            size: 0,
            changelog: std::ptr::null_mut(),
        };

        let code = unsafe { (self.functions.update_check_lite)(&mut info) };
        if code != 0 {
            // 错误分支也需释放可能已分配的 FFI 内存
            unsafe { (self.functions.update_free_info_lite)(&mut info) };
            return Err(SdkError::FfiFailed(code));
        }

        let result = UpdateInfoLite::from_ffi(&info);

        // 释放 FFI 内存
        unsafe { (self.functions.update_free_info_lite)(&mut info) };

        Ok(result)
    }

    /// 获取最后错误信息
    pub fn last_error(&self) -> Option<String> {
        let error_ptr = unsafe { (self.functions.last_error)() };
        if error_ptr.is_null() {
            return None;
        }

        let error_ref = unsafe { &*error_ptr };
        let message = if error_ref.message.is_null() {
            String::new()
        } else {
            unsafe { std::ffi::CStr::from_ptr(error_ref.message) }
                .to_string_lossy()
                .to_string()
        };

        // 释放错误对象
        unsafe { (self.functions.free_error)(error_ptr as *mut _) };

        Some(message)
    }

    /// 清除错误
    pub fn clear_error(&self) {
        unsafe { (self.functions.clear_error)() };
    }
}

impl Drop for SdkInstance {
    fn drop(&mut self) {
        log_info!("SDK lite instance dropped");
    }
}
