//! Mod 数据类型
//! 包含：
//! - ModInfo：list_mods 命令返回的单个 Mod 信息（前端消费）
//! - ModMeta：jar 内 mod 元数据中间结构（仅 mods 子模块内部使用）
//! - ModMetadata：定义已下沉至 minecraft::community::types，本模块 re-export 保持旧路径

use serde::{Deserialize, Serialize};

/// 单个 Mod 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    /// 文件名（不含路径，含扩展名）
    pub file_name: String,
    /// 启用时的文件名（去除 .disabled / .old 后缀）
    pub enabled_name: String,
    /// 是否启用
    pub is_enabled: bool,
    /// 文件大小（字节）
    pub size: u64,
    /// Mod 加载器类型（forge/fabric/neoforge/liteloader/unknown）
    /// 通过文件名和扩展名推断，简化处理
    pub loader_type: String,
    /// 中文译名（来自 mcmod 数据库，可能为空）
    /// 由 community_mod_local_name_style 控制在 UI 中的显示方式：
    ///   0 = 标题显示译名，详情显示文件名
    ///   1 = 标题显示文件名，详情显示译名
    pub translated_name: String,
    /// Mod 描述（来自 jar 内 metadata，可能为空）
    /// fabric.mod.json 的 description / mods.toml 的 description / mcmod.info 的 description
    #[serde(default)]
    pub description: String,
    /// Mod 版本号（来自 jar 内 metadata，可能为空）
    /// fabric.mod.json 的 version / mods.toml 的 version / mcmod.info 的 version
    #[serde(default)]
    pub version: String,
    /// Mod slug（来自 jar 内 metadata：fabric.mod.json 的 id / mods.toml 的 modId / mcmod.info 的 modid）
    /// 用于「详情」按钮关联 CF/MR 平台工程和「前往百科」按钮查 mcmod.cn 直链
    #[serde(default)]
    pub slug: String,
}

/// jar 内 mod metadata 中间结构（仅在 mods 子模块内部使用，由 read_fabric_mod_meta 等填充，finalize_metadata 转换为 ModMetadata）
pub(super) struct ModMeta {
    pub slug: Option<String>,
    pub description: String,
    pub version: String,
    /// 依赖的 mod_id 列表（由 MetaBuilder 累积合并，传给 ModMetadata）
    pub dependencies: Vec<String>,
}
