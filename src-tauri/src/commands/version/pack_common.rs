//! 版本内容目录通用操作（mods / resourcepacks / shaderpacks 共用）
//! 提供目录解析、条目枚举、启停、删除、安装、原子更新、目录监听。

mod entries;
mod resolve;
mod update;

pub(crate) use entries::{
    delete_entry, enabled_name_of, install_entry, list_entries, toggle_entry,
};
pub(crate) use resolve::{resolve_effective_game_dir, resolve_version_subdir};
pub(crate) use update::{download_and_replace, unwatch_dir, watch_dir};
