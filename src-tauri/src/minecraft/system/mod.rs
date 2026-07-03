//! 系统信息获取模块

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