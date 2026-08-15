//! 离线账号皮肤资源包生成模块（方案 B）
//!
//! 生成资源包 zip 替换原版玩家纹理，让离线账号自定义皮肤生效（与方案 A 互补）。
//! 子模块：generate（资源包生成）/ install（启用与移除）。

mod generate;
mod install;

pub(crate) use generate::get_pack_format;
pub use install::{apply_skin_resourcepack, remove_skin_resourcepack};
