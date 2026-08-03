//! 配置数据类型：补丁、快照、条目
//!
//! - `ConfigPatch`：`apply_config` 入参，所有字段 `Option<T>`，仅传需要改的字段（`patch`）
//! - `ConfigSnapshot`：`get_config` 返回的全量配置快照（`snapshot`）
//! - `ConfigEntry` + `build_snapshot`：扁平化条目与快照构建（`entry`）

mod entry;
mod patch;
mod snapshot;

pub use entry::{build_snapshot, ConfigEntry};
pub use patch::ConfigPatch;
pub use snapshot::ConfigSnapshot;
