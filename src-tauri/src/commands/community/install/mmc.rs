//! 社区资源下载安装 - MultiMC 整合包数据结构
//!
//! 仅包含 MMC mmc-pack.json 的反序列化结构。
//! MMC 整合包不含依赖 mods 列表（mods 已打包在 overrides 的 .minecraft/mods/ 中），
//! 通过 components[] 数组指定 Minecraft 本体与加载器版本。
//! 参考 PCL2 ModModpack.vb InstallPackMMC 实现。
//!
//! instance.cfg 中的 `name=xxx` 字段用于实例名默认值，暂不在后端解析
//! （前端弹窗输入实例名，后端只负责解析 mmc-pack.json）。

use serde::Deserialize;

/// MMC 整合包 mmc-pack.json 结构
#[derive(Debug, Deserialize)]
pub(super) struct MmcPack {
    /// 组件列表（net.minecraft / net.minecraftforge / net.fabricmc.fabric-loader 等）
    #[serde(default)]
    pub(super) components: Vec<MmcComponent>,
}

/// MMC 组件
///
/// 常见 uid：
/// - `net.minecraft`：Minecraft 本体，version 为游戏版本
/// - `net.minecraftforge`：Forge，version 为 Forge 版本
/// - `net.neoforged`：NeoForge，version 为 NeoForge 版本
/// - `net.fabricmc.fabric-loader`：Fabric，version 为 Fabric Loader 版本
/// - `org.lwjgl.*`：LWJGL，跳过
#[derive(Debug, Deserialize)]
pub(super) struct MmcComponent {
    #[serde(default)]
    pub(super) uid: String,
    #[serde(default)]
    pub(super) version: String,
}
