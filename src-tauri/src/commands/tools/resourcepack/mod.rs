//! 资源包管理
//! - `list`：列出 resourcepacks 目录下顶层条目（.zip 文件 / 目录）
//!   - 默认扫全局 `{game_dir}/resourcepacks/`
//!   - 传入 `version_id` 时按版本隔离配置解析该版本的有效游戏目录
//! - `convert`：在 zip 与 folder 格式之间转换（folder → 打包为同名 .zip；zip → 解压为同名目录）
//!   子模块：list（列目录）/ convert（格式转换）

mod convert;
mod helpers;
mod list;

pub use convert::convert;
pub use list::list;

// 私有 use：保持子模块 `use super::{resolve_packs_dir, path_to_string};` 引用可用
use helpers::{path_to_string, resolve_packs_dir};