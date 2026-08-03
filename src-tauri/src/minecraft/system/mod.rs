//! 系统信息获取模块

pub mod shell;
mod info;

pub use info::{
    get_system_arch, get_system_memory, get_os_type, is_64bit_system, suggest_memory,
    suggest_memory_from_available, SystemMemory,
};
