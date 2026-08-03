//! 系统信息获取模块

mod info;
pub mod shell;

pub use info::{
    get_os_type, get_system_arch, get_system_memory, is_64bit_system, suggest_memory,
    suggest_memory_from_available, SystemMemory,
};
