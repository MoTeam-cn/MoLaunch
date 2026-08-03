//! Storage module - manages .Molaunch folder
//! All operations on .Molaunch folder must go through this module
//! Uses INI format for configuration
//! 子模块：manager（单例 + 初始化）/ paths（路径解析）/ fs（INI 读写）/ appdata（AppData 共享目录）

mod fs;
mod manager;
mod paths;

pub mod appdata;
pub mod cache;
pub mod cache_app;
pub mod cache_temp;
pub mod ini;
pub mod registry;

pub use manager::Storage;
