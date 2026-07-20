//! 版本 Setup 数据结构
//!
//! PersonalizationUpdate（更新补丁）与 VersionSetup（完整快照）。
//!
//! 为便于理解与后续重构，将 VersionSetup 的 30 个字段按职责分为 4 组，
//! 用 `LoaderInfo` / `DisplayConfig` / `JavaConfig` / `AdvancedConfig` 四个子 struct
//! 类型化表示。当前 VersionSetup 仍保持平铺字段以兼容现有访问代码，
//! 子 struct 作为分组视图供未来重构使用。

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

/// 加载器与版本基础信息（VersionSetup 字段分组视图：8 字段）
///
/// 当前作为类型化的字段分组存在，便于未来将 VersionSetup 重构为嵌套结构。
/// 注意：未 derive Default，因为 VersionType 未实现 Default。
#[allow(dead_code)]
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

/// 显示与分类配置（VersionSetup 字段分组视图：7 字段）
#[allow(dead_code)]
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

/// Java 与内存配置（VersionSetup 字段分组视图：7 字段）
#[allow(dead_code)]
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

/// 高级选项配置（VersionSetup 字段分组视图：8 字段）
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct AdvancedConfig {
    pub advance_jvm_args: Option<String>,
    pub advance_game_args: Option<String>,
    pub advance_run_cmd: Option<String>,
    pub advance_disable_mod_update: Option<bool>,
    pub advance_ignore_java_warning: Option<bool>,
    pub advance_disable_assets_verify: Option<bool>,
    pub advance_disable_jlw: Option<bool>,
    pub advance_disable_lua: Option<bool>,
}

/// 版本 Setup 信息
#[derive(Debug, Clone)]
pub struct VersionSetup {
    // ===== LoaderInfo 分组 =====
    /// 原始 MC 版本号（如 1.20.1）
    pub original_version: String,
    /// 版本类型
    pub version_type: VersionType,
    /// Forge 版本号（如有）
    pub forge_version: Option<String>,
    /// NeoForge 版本号（如有）
    pub neoforge_version: Option<String>,
    /// Fabric Loader 版本号（如有）
    pub fabric_version: Option<String>,
    /// Quilt Loader 版本号（如有）
    pub quilt_version: Option<String>,
    /// OptiFine 版本号（如有）
    pub optifine_version: Option<String>,
    /// LiteLoader 版本号（如有）
    pub liteloader_version: Option<String>,
    // ===== DisplayConfig 分组 =====
    /// 自定义图标文件名（空字符串=自动判断，logo.png 等相对路径）
    pub logo: Option<String>,
    /// 自定义描述（空字符串=使用默认描述）
    pub custom_info: Option<String>,
    /// 强制版本分类（0=自动，1=隐藏，2=可安装Mod，3=原版类似，4=垃圾，5=愚人节，6=错误）
    pub display_type: Option<i32>,
    /// 是否收藏
    pub is_star: Option<bool>,
    /// 版本独立隔离设置（0=跟随全局，1=开启隔离，2=关闭隔离）
    pub indie_type: Option<i32>,
    /// 游戏窗口标题（空=跟随全局）
    pub window_title: Option<String>,
    /// 自动进入服务器（"IP:Port" 格式，空=不自动进入）
    pub server_enter: Option<String>,
    // ===== AdvancedConfig 分组 =====
    /// 额外 JVM 参数（空=跟随全局）
    pub advance_jvm_args: Option<String>,
    /// 额外游戏参数（空=跟随全局）
    pub advance_game_args: Option<String>,
    /// 启动前执行命令（空=跟随全局）
    pub advance_run_cmd: Option<String>,
    // ===== JavaConfig 分组 =====
    /// 版本独立 Java 路径（仅 JavaMode="custom" 时生效）
    pub java_path: Option<String>,
    /// Java 选择模式：None/空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java
    pub java_mode: Option<String>,
    /// 自动选择时的最小 Java 主版本（仅 JavaMode="auto_version" 时生效，0=不限）
    pub java_version_min: Option<u32>,
    /// 自动选择时的最大 Java 主版本（仅 JavaMode="auto_version" 时生效，0=不限）
    pub java_version_max: Option<u32>,
    /// 内存模式：None/空=跟随全局, "auto"=自动, "custom"=自定义
    pub memory_mode: Option<String>,
    /// 版本独立最小内存（MB，仅 custom 模式生效）
    pub min_memory: Option<u32>,
    /// 版本独立最大内存（MB，仅 custom 模式生效）
    pub max_memory: Option<u32>,
    // ===== 高级选项开关 =====
    /// 禁止更新 Mod
    pub advance_disable_mod_update: Option<bool>,
    /// 忽略 Java 兼容性警告
    pub advance_ignore_java_warning: Option<bool>,
    /// 关闭文件校验
    pub advance_disable_assets_verify: Option<bool>,
    /// 禁用 Java Launch Wrapper
    pub advance_disable_jlw: Option<bool>,
    /// 禁用 LWJGL Unsafe Agent
    pub advance_disable_lua: Option<bool>,
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
            original_version: original_version.to_string(),
            version_type,
            forge_version: forge.map(|s| s.to_string()),
            neoforge_version: neoforge.map(|s| s.to_string()),
            fabric_version: fabric.map(|s| s.to_string()),
            quilt_version: quilt.map(|s| s.to_string()),
            optifine_version: optifine.map(|s| s.to_string()),
            liteloader_version: liteloader.map(|s| s.to_string()),
            logo: None,
            custom_info: None,
            display_type: None,
            is_star: None,
            indie_type: None,
            window_title: None,
            server_enter: None,
            advance_jvm_args: None,
            advance_game_args: None,
            advance_run_cmd: None,
            java_path: None,
            java_mode: None,
            java_version_min: None,
            java_version_max: None,
            memory_mode: None,
            min_memory: None,
            max_memory: None,
            advance_disable_mod_update: None,
            advance_ignore_java_warning: None,
            advance_disable_assets_verify: None,
            advance_disable_jlw: None,
            advance_disable_lua: None,
        }
    }

    /// 全空默认 Setup（用于 setup.ini 不存在时的兜底）
    pub fn empty() -> Self {
        Self {
            original_version: String::new(),
            version_type: VersionType::Unknown,
            forge_version: None,
            neoforge_version: None,
            fabric_version: None,
            quilt_version: None,
            optifine_version: None,
            liteloader_version: None,
            logo: None,
            custom_info: None,
            display_type: None,
            is_star: None,
            indie_type: None,
            window_title: None,
            server_enter: None,
            advance_jvm_args: None,
            advance_game_args: None,
            advance_run_cmd: None,
            java_path: None,
            java_mode: None,
            java_version_min: None,
            java_version_max: None,
            memory_mode: None,
            min_memory: None,
            max_memory: None,
            advance_disable_mod_update: None,
            advance_ignore_java_warning: None,
            advance_disable_assets_verify: None,
            advance_disable_jlw: None,
            advance_disable_lua: None,
        }
    }
}
