//! 社区资源下载安装 - HMCL 整合包数据结构
//!
//! 仅包含 HMCL modpack.json 的反序列化结构。
//! HMCL 整合包不含依赖 mods 列表（mods 已打包在 overrides 的 minecraft/mods/ 中），
//! 安装流程只需解压 `minecraft/` overrides 到 instance 目录，再安装游戏本体。
//! 参考 PCL2 ModModpack.vb InstallPackHMCL 实现。

use serde::Deserialize;

/// HMCL 整合包 modpack.json 结构
///
/// 关键字段：
/// - `gameVersion`：Minecraft 版本号（必需）
/// - `name`：整合包名称（可选，作为实例名默认值）
///
/// 其他字段（如 `author`、`description`、`version`、`files`）暂不使用，
/// PCL2 也仅读取 `gameVersion` 和 `name`。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HmclManifest {
    /// Minecraft 版本号（如 "1.20.1"）
    pub(super) game_version: String,
    /// 整合包名称（作为实例名默认值）
    #[serde(default)]
    pub(super) name: String,
}
