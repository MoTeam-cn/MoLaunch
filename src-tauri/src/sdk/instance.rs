//! SDK 实例和函数集合定义

use super::ffi_types::*;
use super::types::*;
use super::{check_sdk_library, SdkError};

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
    #[allow(clippy::too_many_arguments)]
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
            max_download_speed,  // ← 修正：先 u64
            mirror_mode,         // ← 修正：后 u32
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
    pub fn version(&self) -> Result<String, SdkError> {
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }
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
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }
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
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }
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
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }
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
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }
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
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }
        let code = unsafe { (self.functions.reset_progress)() };
        if code != 0 {
            return Err(SdkError::FfiFailed(code));
        }
        Ok(())
    }

    /// 检查是否正在下载
    pub fn is_downloading(&self) -> Result<bool, SdkError> {
        if self.handle.is_null() {
            return Err(SdkError::NotInitialized);
        }
        Ok(unsafe { (self.functions.is_downloading)() == 1 })
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
        let forge_c = forge_version.map(|v| std::ffi::CString::new(v).map_err(|e| SdkError::InvalidParameter(e.to_string()))).transpose()?;
        let neoforge_c = neoforge_version.map(|v| std::ffi::CString::new(v).map_err(|e| SdkError::InvalidParameter(e.to_string()))).transpose()?;
        let fabric_c = fabric_version.map(|v| std::ffi::CString::new(v).map_err(|e| SdkError::InvalidParameter(e.to_string()))).transpose()?;
        let optifine_c = optifine_version.map(|v| std::ffi::CString::new(v).map_err(|e| SdkError::InvalidParameter(e.to_string()))).transpose()?;
        let liteloader_c = liteloader_version.map(|v| std::ffi::CString::new(v).map_err(|e| SdkError::InvalidParameter(e.to_string()))).transpose()?;
        let instance_c = instance_name.map(|v| std::ffi::CString::new(v).map_err(|e| SdkError::InvalidParameter(e.to_string()))).transpose()?;

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
