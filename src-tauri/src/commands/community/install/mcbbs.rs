//! 社区资源下载安装 - MCBBS 整合包数据结构
//!
//! 仅包含 MCBBS mcbbs.packmeta（或带 addons 的 manifest.json）的反序列化结构。
//! MCBBS 整合包不含依赖 mods 列表（mods 已打包在 overrides/mods/ 中），
//! 通过 addons[] 数组指定 Minecraft 本体与加载器版本。
//! 参考 PCL2 ModModpack.vb InstallPackMCBBS 实现。

use serde::Deserialize;

/// MCBBS 整合包 mcbbs.packmeta / manifest.json 结构
///
/// 关键字段：
/// - `addons`：版本附加信息数组（id+version 对，包含 game/forge/fabric/neoforge/optifine/quilt）
/// - `name`：整合包名称（可选，作为实例名默认值）
/// - `launchInfo`：启动参数（javaArgument / launchArgument，暂不使用）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct McbbsManifest {
    /// 版本附加信息数组
    #[serde(default)]
    pub(super) addons: Vec<McbbsAddon>,
    /// 整合包名称（作为实例名默认值）
    #[serde(default)]
    pub(super) name: String,
}

/// MCBBS 版本附加项
///
/// 常见 id：
/// - `game`：Minecraft 版本（必需）
/// - `forge`：Forge 版本
/// - `neoforge`：NeoForge 版本
/// - `fabric`：Fabric Loader 版本
/// - `optifine`：OptiFine 版本
/// - `quilt`：Quilt 版本（PCL2 不支持，会拒绝安装）
#[derive(Debug, Deserialize)]
pub(super) struct McbbsAddon {
    #[serde(default)]
    pub(super) id: String,
    #[serde(default)]
    pub(super) version: String,
}
