//! 本地 Mod 详情预加载
//! `list_mods` 同步只做文件枚举（瞬间返回），本模块异步补全元数据：
//! 1. 读 JAR 元数据（slug / 描述 / 版本 / logo / 译名）
//! 2. 批量 hash 查询 CF/MR 工程详情
//!    通过 `mods-preload-update` 事件推送前端，持久化缓存写入 `.Molaunch/cache/preload_mods/{version_id}.json`（6h TTL）

mod api;
mod cache;
mod hash;
mod jar_metadata;
mod online_query;
mod types;

pub use api::preload_mods_detail;
pub use hash::{compute_curseforge_fingerprint, compute_modrinth_sha1};
pub use types::{PreloadModInput, PreloadUpdate};