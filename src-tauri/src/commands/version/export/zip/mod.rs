//! Zip 打包分发入口（按 ExportFormat 分发到 6 种格式 builder）
//!
//! 分发逻辑在 `api`，共享辅助（zip I/O、版本依赖解析）位于 `helpers`；
//! 各格式实现在 `modrinth` / `curseforge` / `hmcl` / `mmc` / `mcbbs` / `compress`。

mod api;
mod compress;
mod curseforge;
mod helpers;
mod hmcl;
mod mcbbs;
mod mmc;
mod modrinth;

pub use api::build_modpack_zip;
