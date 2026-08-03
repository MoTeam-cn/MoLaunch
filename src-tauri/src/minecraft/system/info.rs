//! 系统信息获取
//!
//! 内存信息（SystemMemory / suggest_memory）与平台信息（arch / os / 64bit）。

use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt};

/// 系统内存信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMemory {
    /// 总内存（字节）
    pub total: u64,
    /// 已用内存（字节）
    pub used: u64,
    /// 可用内存（字节）
    pub available: u64,
    /// 内存使用率（百分比）
    pub usage_percent: f64,
}

/// 获取系统内存信息
pub fn get_system_memory() -> SystemMemory {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let available = total - used;
    let usage_percent = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    SystemMemory {
        total,
        used,
        available,
        usage_percent,
    }
}

/// 获取系统架构
pub fn get_system_arch() -> String {
    std::env::consts::ARCH.to_string()
}

/// 获取操作系统类型
pub fn get_os_type() -> String {
    std::env::consts::OS.to_string()
}

/// 检查是否为64位系统
pub fn is_64bit_system() -> bool {
    std::mem::size_of::<usize>() == 8
}

/// 根据系统可用内存推算 Minecraft 推荐内存配置（auto 模式统一算法）。
/// 返回 `(min_mb, max_mb)`：max = min(可用*0.75, 8192) 且 >= 512，min = max/2。
pub fn suggest_memory() -> (u32, u32) {
    let sys_mem = get_system_memory();
    let available_mb = (sys_mem.available / 1024 / 1024) as u32;
    suggest_memory_from_available(available_mb)
}

/// 根据可用内存（MB）推算推荐内存配置。
pub fn suggest_memory_from_available(available_mb: u32) -> (u32, u32) {
    let suggested_max = std::cmp::min((available_mb as f64 * 0.75) as u32, 8192);
    let suggested_max = std::cmp::max(suggested_max, 512);
    let suggested_min = suggested_max / 2;
    (suggested_min, suggested_max)
}
