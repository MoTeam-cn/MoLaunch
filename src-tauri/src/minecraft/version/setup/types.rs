//! 版本 Setup 数据结构
//!
//! PersonalizationUpdate（更新补丁）与 VersionSetup（完整快照）。
//!
//! VersionSetup 按职责拆分为 4 个嵌套子 struct：
//! `LoaderInfo`（加载器信息）/ `DisplayConfig`（显示配置）/
//! `JavaConfig`（Java 与内存）/ `AdvancedConfig`（高级选项）。

use super::super::state::VersionType;
use serde::{Deserialize, Serialize};

/// 版本个性化字段更新（所有字段可选，None 表示不修改）
/// 注意：前端传 camelCase（如 javaPath），需 rename_all 匹配
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalizationUpdate {
    pub logo: Option<String>,
    pub custom_info: Option<String>,
    pub display_type: Option<i32>,
    pub is_star: Option<bool>,
    pub indie_type: Option<i32>,
    pub window_title: Option<String>,
    pub server_enter: Option<String>,
    pub advance_jvm_args: Option<String>,
    pub advance_game_args: Option<String>,
    pub advance_run_cmd: Option<String>,
    pub java_path: Option<String>,
    /// Java 选择模式：None/空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java
    pub java_mode: Option<String>,
    /// 自动选择时的最小 Java 主版本（仅 auto_version 模式生效，0=不限）
    pub java_version_min: Option<u32>,
    /// 自动选择时的最大 Java 主版本（仅 auto_version 模式生效，0=不限）
    pub java_version_max: Option<u32>,
    /// 内存模式：None=跟随全局, Some("auto")=自动, Some("custom")=自定义
    pub memory_mode: Option<String>,
    /// 版本独立最小内存（MB，仅 custom 模式生效）
    pub min_memory: Option<u32>,
    /// 版本独立最大内存（MB，仅 custom 模式生效）
    pub max_memory: Option<u32>,
    // ===== 高级选项开关 =====
    /// 禁止更新 Mod（None=跟随全局默认 false）
    pub advance_disable_mod_update: Option<bool>,
    /// 忽略 Java 兼容性警告（None=跟随全局默认 false）
    pub advance_ignore_java_warning: Option<bool>,
    /// 关闭文件校验（None=跟随全局默认 false）
    pub advance_disable_assets_verify: Option<bool>,
    /// 禁用 Java Launch Wrapper（None=跟随全局默认 false）
    pub advance_disable_jlw: Option<bool>,
    /// 禁用 LWJGL Unsafe Agent（None=跟随全局默认 false）
    pub advance_disable_lua: Option<bool>,
}

/// 加载器与版本基础信息（VersionSetup.loader：8 字段）
///
/// 注意：未 derive Default，因为 VersionType 未实现 Default。
#[derive(Debug, Clone)]
pub struct LoaderInfo {
    pub original_version: String,
    pub version_type: VersionType,
    pub forge_version: Option<String>,
    pub neoforge_version: Option<String>,
    pub fabric_version: Option<String>,
    pub quilt_version: Option<String>,
    pub optifine_version: Option<String>,
    pub liteloader_version: Option<String>,
}

/// 显示与分类配置（VersionSetup.display：7 字段）
#[derive(Debug, Clone, Default)]
pub struct DisplayConfig {
    pub logo: Option<String>,
    pub custom_info: Option<String>,
    pub display_type: Option<i32>,
    pub is_star: Option<bool>,
    pub indie_type: Option<i32>,
    pub window_title: Option<String>,
    pub server_enter: Option<String>,
}

/// Java 与内存配置（VersionSetup.java：7 字段）
#[derive(Debug, Clone, Default)]
pub struct JavaConfig {
    pub java_path: Option<String>,
    pub java_mode: Option<String>,
    pub java_version_min: Option<u32>,
    pub java_version_max: Option<u32>,
    pub memory_mode: Option<String>,
    pub min_memory: Option<u32>,
    pub max_memory: Option<u32>,
}

/// 高级选项配置（VersionSetup.advanced：8 字段）
#[derive(Debug, Clone, Default)]
pub struct AdvancedConfig {
    pub jvm_args: Option<String>,
    pub game_args: Option<String>,
    pub run_cmd: Option<String>,
    pub disable_mod_update: Option<bool>,
    pub ignore_java_warning: Option<bool>,
    pub disable_assets_verify: Option<bool>,
    pub disable_jlw: Option<bool>,
    pub disable_lua: Option<bool>,
}

/// 版本 Setup 信息
#[derive(Debug, Clone)]
pub struct VersionSetup {
    pub loader: LoaderInfo,
    pub display: DisplayConfig,
    pub java: JavaConfig,
    pub advanced: AdvancedConfig,
}

impl VersionSetup {
    /// 创建新的 Setup（安装时调用）
    pub fn new(
        original_version: &str,
        version_type: VersionType,
        forge: Option<&str>,
        neoforge: Option<&str>,
        fabric: Option<&str>,
        quilt: Option<&str>,
        optifine: Option<&str>,
        liteloader: Option<&str>,
    ) -> Self {
        Self {
            loader: LoaderInfo {
                original_version: original_version.to_string(),
                version_type,
                forge_version: forge.map(|s| s.to_string()),
                neoforge_version: neoforge.map(|s| s.to_string()),
                fabric_version: fabric.map(|s| s.to_string()),
                quilt_version: quilt.map(|s| s.to_string()),
                optifine_version: optifine.map(|s| s.to_string()),
                liteloader_version: liteloader.map(|s| s.to_string()),
            },
            display: DisplayConfig::default(),
            java: JavaConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }

    /// 全空默认 Setup（用于 setup.ini 不存在时的兜底）
    pub fn empty() -> Self {
        Self {
            loader: LoaderInfo {
                original_version: String::new(),
                version_type: VersionType::Unknown,
                forge_version: None,
                neoforge_version: None,
                fabric_version: None,
                quilt_version: None,
                optifine_version: None,
                liteloader_version: None,
            },
            display: DisplayConfig::default(),
            java: JavaConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}
