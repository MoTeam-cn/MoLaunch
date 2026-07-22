//! 版本 Mod 管理命令
//!
//! 模块结构：
//! - types.rs: ModInfo / ModMetadata / ModMeta 数据类型
//! - helpers.rs: get_mods_dir 共享辅助函数（sanitize_file_name 已迁移到 utils::path）
//! - metadata.rs: jar 内 mod 元数据读取流水线（read_mod_metadata + 8 个内部辅助）
//! - watcher.rs: mods 目录文件监听（notify crate + 防抖 + emit mods-dir-changed 事件）
//! - list.rs: 列表查询命令（list_mods / is_version_modable + infer_loader_type）
//! - manage.rs: 管理命令（toggle_mod / delete_mod）
//! - install.rs: 安装与文件操作命令（install_mod / open_mods_dir / get_version_mods_dir / reveal_mod_file）
//! - mod.rs: 模块入口 + pub mod 声明 + 类型 re-export
//!
//! 注意：所有 #[tauri::command] 命令分散在 list/manage/install/watcher 子模块中，
//! tauri::command 宏在定义处生成 __cmd__ 符号，不能通过 pub use 重导出，
//! 故 lib.rs 使用完整路径注册（commands::version::mods::list::* / ::manage::* / ::install::* / ::watcher::*）

pub(crate) mod helpers;
pub mod install;
pub mod list;
mod metadata;
pub mod manage;
mod types;
pub mod watcher;

// 对外暴露类型和辅助函数（保持向后兼容路径）
// 注意：ModMetadata 在 metadata.rs 中是私有 use 引入的（use super::types::ModMetadata），
// 故必须从 types 直接重导出，不能走 metadata 中转
pub(crate) use helpers::get_mods_dir;
pub(crate) use metadata::read_mod_metadata;
pub use types::ModInfo;
pub(crate) use types::ModMetadata;
