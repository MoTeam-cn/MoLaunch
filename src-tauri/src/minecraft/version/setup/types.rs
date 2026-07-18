//! 版本 Setup 数据结构
//!
//! PersonalizationUpdate（更新补丁）与 VersionSetup（完整快照）。

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
    // ===== 高级选项开关（参考 PCL2 PageInstanceSetup 高级选项）=====
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

/// 版本 Setup 信息
#[derive(Debug, Clone)]
pub struct VersionSetup {
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
    /// 自定义图标文件名（空字符串=自动判断，PCL\Logo.png 等相对路径）
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
    /// 额外 JVM 参数（空=跟随全局）
    pub advance_jvm_args: Option<String>,
    /// 额外游戏参数（空=跟随全局）
    pub advance_game_args: Option<String>,
    /// 启动前执行命令（空=跟随全局）
    pub advance_run_cmd: Option<String>,
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
    // ===== 高级选项开关（参考 PCL2 PageInstanceSetup 高级选项）=====
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
