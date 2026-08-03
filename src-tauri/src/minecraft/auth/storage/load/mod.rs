//! `AuthStorage::load` 实现：注册表 / JSON 文件双轨制读取（优先返回内存缓存）
//!
//! 子模块：file（非 Windows JSON）/ registry（Windows 注册表）/ router（读取分发）。

#[cfg(not(windows))]
mod file;
#[cfg(windows)]
mod registry;
mod router;