//! 统一配置更新命令
//! 单个 `apply_config` IPC 取代此前分散在 proxy/download/game/community 等模块的 19 个
//! `set_*` setter。前端传 `ConfigPatch`（全 `Option<T>`，仅传需改字段）一次性多字段更新，
//! 后端在单次 `update_config` 闭包内完成赋值与联动。三段式分流：校验（mirror_url SSRF 防护、
//! download_source/meta_source 枚举校验）→ 加密字段分流 → 普通字段统一更新。

mod apply;
mod dispatcher;
mod secure;
mod types;
mod validate;

pub use dispatcher::{apply_config, get_config};
pub use types::{ConfigEntry, ConfigPatch, ConfigSnapshot};