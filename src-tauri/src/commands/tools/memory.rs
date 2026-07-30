//! 内存优化（双模式）
//!
//! 释放系统占用的物理内存，降低系统资源消耗。提供两种模式：
//!
//! ## 优化模式
//! - **light（轻量）**：仅清空所有进程的工作集。
//!   调用 `NtSetSystemInformation(SystemMemoryListInformation, MemoryEmptyWorkingSets)`，
//!   一次系统调用即可清空所有进程工作集，无需遍历进程。
//!   释放量较小（几十 MB ~ 几百 MB），但响应迅速、几乎无副作用。
//!
//! - **strong（强力）**：依次执行 4 个系统内存操作，释放数 GB：
//!   1. `MemoryFlushModifiedList`：将已修改页面写入磁盘（释放 dirty page）
//!   2. `MemoryPurgeLowPriorityStandbyList`：清理低优先级待机列表
//!   3. `MemoryPurgeStandbyList`：清理待机列表（standby list，关键释放源）
//!   4. `MemoryEmptyWorkingSets`：清空所有进程工作集
//!
//!   ⚠️ 强力模式会清空 standby list，可能导致已缓存的应用下次启动变慢。
//!
//! ## 平台实现
//! - **Windows**：通过 `NtSetSystemInformation` + `SystemMemoryListInformation`
//!   执行系统级内存操作。
//! - **Linux**：调用 glibc 的 `malloc_trim(0)` 归还堆碎片给 OS。
//! - **macOS**：调用 `malloc_zone_pressure_relief(NULL, 0)` 释放所有 malloc zone 的空闲内存。
//!
//! 内存采样：用 sysinfo 获取系统可用内存（before/after 差值即为释放量）。

use sysinfo::{System, SystemExt};

use crate::commands::tools::types::{MemoryOptimizeParams, MemoryOptimizeResult};
use crate::log_info;

/// 优化内存：根据 mode 执行轻量或强力内存释放
pub async fn optimize(params: MemoryOptimizeParams) -> Result<serde_json::Value, String> {
    let mode = if params.mode == "strong" { "strong" } else { "light" };

    // 1. 优化前可用内存（字节）
    let before = get_available_memory_bytes();

    // 2. 平台相关的内存释放
    release_memory(mode);

    // 3. 短暂 sleep 500ms 让 OS 完成回收
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 4. 优化后可用内存（字节）
    let after = get_available_memory_bytes();

    let result = MemoryOptimizeResult {
        freed_bytes: after.saturating_sub(before),
        before_bytes: before,
        after_bytes: after,
        mode: mode.to_string(),
    };

    log_info!(
        "[MemoryOptimize] mode={} before={}bytes after={}bytes freed={}bytes",
        result.mode,
        result.before_bytes,
        result.after_bytes,
        result.freed_bytes
    );

    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 获取系统可用内存（字节）
///
/// sysinfo 0.29 文档声明返回 KB，但实际在某些平台/版本上返回字节。
/// 通过 `total_memory()` 的量级判断单位：
/// - 如果 total > 1,000,000,000（>1TB KB 不可能出现在普通 PC），说明返回的是字节
/// - 否则返回 KB，需要 × 1024 转为字节
fn get_available_memory_bytes() -> u64 {
    let mut sys = System::new();
    sys.refresh_memory();

    let total = sys.total_memory();
    let avail = sys.available_memory();

    if total > 1_000_000_000 {
        // sysinfo 返回字节（total > 1TB KB 不可能出现在普通 PC）
        avail
    } else {
        // sysinfo 返回 KB，转为字节
        avail * 1024
    }
}

#[cfg(target_os = "windows")]
mod nt {
    /// SYSTEM_INFORMATION_CLASS 中的 SystemMemoryListInformation（值 = 80）
    pub const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 80;

    /// SYSTEM_MEMORY_LIST_COMMAND 枚举
    /// https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ne-wdm-_system_memory_list_command
    #[repr(i32)]
    #[allow(non_camel_case_types)]
    pub enum SystemMemoryListCommand {
        /// 清空所有进程的工作集
        MemoryEmptyWorkingSets = 0,
        /// 将已修改页面写入磁盘
        MemoryFlushModifiedList = 1,
        /// 清理低优先级待机列表
        MemoryPurgeLowPriorityStandbyList = 2,
        /// 清理待机列表（释放数 GB 的关键）
        MemoryPurgeStandbyList = 3,
    }

    // ntdll.dll 中的 NtSetSystemInformation 未公开 API
    //
    // NTSTATUS NtSetSystemInformation(
    //     IN SYSTEM_INFORMATION_CLASS SystemInformationClass,
    //     IN PVOID SystemInformation,
    //     IN ULONG SystemInformationLength
    // );
    extern "system" {
        pub fn NtSetSystemInformation(
            system_information_class: u32,
            system_information: *const std::ffi::c_void,
            system_information_length: u32,
        ) -> i32;
    }

    /// 调用 NtSetSystemInformation 传入一个 SYSTEM_MEMORY_LIST_COMMAND
    ///
    /// 返回 NTSTATUS（0 = STATUS_SUCCESS）
    pub fn invoke_memory_list_command(cmd: SystemMemoryListCommand) -> i32 {
        let value = cmd as i32;
        unsafe {
            NtSetSystemInformation(
                SYSTEM_MEMORY_LIST_INFORMATION,
                &value as *const i32 as *const std::ffi::c_void,
                std::mem::size_of::<i32>() as u32,
            )
        }
    }
}

/// 平台相关的内存释放
fn release_memory(mode: &str) {
    // Windows: NtSetSystemInformation + SystemMemoryListInformation
    #[cfg(target_os = "windows")]
    {
        use nt::SystemMemoryListCommand as C;

        let mut status_log: Vec<(&str, i32)> = Vec::new();

        if mode == "strong" {
            // 强力模式：依次执行 4 个操作释放数 GB
            status_log.push((
                "MemoryFlushModifiedList",
                nt::invoke_memory_list_command(C::MemoryFlushModifiedList),
            ));
            status_log.push((
                "MemoryPurgeLowPriorityStandbyList",
                nt::invoke_memory_list_command(C::MemoryPurgeLowPriorityStandbyList),
            ));
            status_log.push((
                "MemoryPurgeStandbyList",
                nt::invoke_memory_list_command(C::MemoryPurgeStandbyList),
            ));
        }

        // 两种模式都清空所有进程工作集
        status_log.push((
            "MemoryEmptyWorkingSets",
            nt::invoke_memory_list_command(C::MemoryEmptyWorkingSets),
        ));

        let log_str = status_log
            .iter()
            .map(|(name, status)| format!("{}={:#x}", name, status))
            .collect::<Vec<_>>()
            .join(", ");
        log_info!("[MemoryOptimize] mode={} 调用结果: {}", mode, log_str);
    }

    // Linux: malloc_trim(0) 归还 glibc 堆碎片给 OS
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn malloc_trim(pad: usize) -> std::os::raw::c_int;
        }
        // pad=0 表示尽可能多地归还空闲内存
        let _ = unsafe { malloc_trim(0) };
        let _ = mode;
    }

    // macOS: malloc_zone_pressure_relief(NULL, 0) 释放所有 malloc zone 空闲内存
    #[cfg(target_os = "macos")]
    {
        // malloc_zone_pressure_relief 的签名：
        //   malloc_zone_t *malloc_zone_pressure_relief(malloc_zone_t *zone, size_t goal);
        // zone=NULL 表示对所有 zone 操作，goal=0 表示尽可能释放
        // 返回值是受影响的 zone 指针（我们不使用）
        //
        // 不直接声明 malloc_zone_t 结构体（它很复杂），用 opaque 指针代替
        extern "C" {
            fn malloc_zone_pressure_relief(
                zone: *mut std::os::raw::c_void,
                goal: usize,
            ) -> *mut std::os::raw::c_void;
        }
        let _ = unsafe { malloc_zone_pressure_relief(std::ptr::null_mut(), 0) };
        let _ = mode;
    }
}
