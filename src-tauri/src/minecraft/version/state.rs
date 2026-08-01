//! 版本状态模块
//!
//! 定义版本类型（正式版/快照/愚人节/远古版/Mod加载器），
//! 提供检测函数和序列化支持。

use crate::minecraft::fools;
use serde::{Deserialize, Serialize};

/// 版本类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionType {
    /// 正式版（原版）
    Release,
    /// 快照版
    Snapshot,
    /// 愚人节版本
    Fool,
    /// 远古版本（2000-2013）
    Old,
    /// 安装了 Forge
    Forge,
    /// 安装了 NeoForge
    NeoForge,
    /// 安装了 Fabric
    Fabric,
    /// 安装了 Quilt
    Quilt,
    /// 安装了 OptiFine
    OptiFine,
    /// 安装了 LiteLoader
    LiteLoader,
    /// 未知（无法识别）
    Unknown,
}

impl VersionType {
    /// 转为字符串（用于持久化）
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Snapshot => "snapshot",
            Self::Fool => "fool",
            Self::Old => "old",
            Self::Forge => "forge",
            Self::NeoForge => "neoforge",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::OptiFine => "optifine",
            Self::LiteLoader => "liteloader",
            Self::Unknown => "unknown",
        }
    }

    #[allow(clippy::should_implement_trait)]
    /// 从字符串解析
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "release" => Self::Release,
            "snapshot" => Self::Snapshot,
            "fool" => Self::Fool,
            "old" => Self::Old,
            "forge" => Self::Forge,
            "neoforge" => Self::NeoForge,
            "fabric" => Self::Fabric,
            "quilt" => Self::Quilt,
            "optifine" => Self::OptiFine,
            "liteloader" => Self::LiteLoader,
            _ => Self::Unknown,
        }
    }

    /// 是否为正式版（非快照/愚人节/远古）
    pub fn is_release(&self) -> bool {
        matches!(
            self,
            Self::Release
                | Self::Forge
                | Self::NeoForge
                | Self::Fabric
                | Self::Quilt
                | Self::OptiFine
                | Self::LiteLoader
        )
    }

    /// 是否为 Mod 加载器版本
    pub fn is_modded(&self) -> bool {
        matches!(
            self,
            Self::Forge
                | Self::NeoForge
                | Self::Fabric
                | Self::Quilt
                | Self::LiteLoader
                | Self::OptiFine
        )
    }

    /// 从版本 JSON 检测版本类型（实例方法，方便调用）
    pub fn detect_from_json(version_id: &str, version_json: &serde_json::Value) -> Self {
        detect_version_type(version_id, version_json)
    }
}

/// 从版本 JSON 和版本 ID 检测版本类型
///
/// 检测优先级：
/// 1. 远古版（发布年份 2000-2013）
/// 2. 愚人节版（fools 模块检测）
/// 3. 快照版（type=snapshot 或 YYwWWa 格式）
/// 4. Mod 加载器（libraries 中的 Maven 坐标）
/// 5. 默认为正式版
pub fn detect_version_type(version_id: &str, version_json: &serde_json::Value) -> VersionType {
    let json_type = version_json["type"].as_str().unwrap_or("");
    let release_time = version_json["releaseTime"].as_str().unwrap_or("");

    // 1. 检查远古版本
    if !release_time.is_empty() && is_old_version(release_time) {
        return VersionType::Old;
    }

    // 2. 检查愚人节版本
    if json_type == "fool" || fools::detect_fool(version_id, json_type, release_time).is_some() {
        return VersionType::Fool;
    }

    // 3. 检查快照版本
    if is_snapshot(version_id, json_type) {
        return VersionType::Snapshot;
    }

    // 4. 检查 Mod 加载器
    if let Some(loader) = detect_loader_from_json(version_json) {
        return loader;
    }

    VersionType::Release
}

/// 从加载器安装参数推断版本类型
///
/// 安装时已知加载器类型，无需解析 JSON
pub fn infer_from_loader(
    forge: Option<&str>,
    neoforge: Option<&str>,
    fabric: Option<&str>,
    quilt: Option<&str>,
    optifine: Option<&str>,
    liteloader: Option<&str>,
) -> VersionType {
    if forge.is_some() {
        return VersionType::Forge;
    }
    if neoforge.is_some() {
        return VersionType::NeoForge;
    }
    if fabric.is_some() {
        return VersionType::Fabric;
    }
    if quilt.is_some() {
        return VersionType::Quilt;
    }
    if optifine.is_some() {
        return VersionType::OptiFine;
    }
    if liteloader.is_some() {
        return VersionType::LiteLoader;
    }
    VersionType::Release
}

/// 从版本 JSON 的 libraries 检测加载器类型
///
/// 字符串包含判断：
/// - 优先级：OptiFine > LiteLoader > Fabric > NeoForge > Forge
/// - 用 JSON 原始字符串包含判断，而不是解析 libraries 数组的 name.starts_with
///   因为新版 Forge（1.20.1+）library 拆分为 fmlloader/jarjar 等多个独立模块，
///   没有 `net.minecraftforge:forge:` 这个 library，但 JSON 内容里仍含 "minecraftforge"
/// - Forge 排除 NeoForge：`minecraftforge` 关键字 NeoForge 的 JSON 也会命中（net.neoforge
///   的 loader 安装时复用了 minecraftforge 命名空间），所以必须先排除
fn detect_loader_from_json(version_json: &serde_json::Value) -> Option<VersionType> {
    // 用原始 JSON 文本做关键字搜索，避免漏掉新版加载器
    let json_text = version_json.to_string();
    let json_lower = json_text.to_lowercase();

    // 检测 OptiFine
    if json_lower.contains("optifine") {
        return Some(VersionType::OptiFine);
    }
    // 检测 LiteLoader
    if json_lower.contains("liteloader") {
        return Some(VersionType::LiteLoader);
    }
    // 检测 Fabric / Quilt（先于 Forge 判断，避免误判）
    if json_lower.contains("net.fabricmc:fabric-loader") {
        return Some(VersionType::Fabric);
    }
    if json_lower.contains("org.quiltmc:quilt-loader") {
        return Some(VersionType::Quilt);
    }
    // 检测 NeoForge（必须在 Forge 之前判断，因为 NeoForge 的 JSON 也含 minecraftforge）
    // 1.20.2+ 的 NeoForge JSON 会有 net.neoforged 命名空间
    if json_lower.contains("net.neoforge") || json_lower.contains("net.neoforged") {
        return Some(VersionType::NeoForge);
    }
    // 检测 Forge
    // 关键字："minecraftforge" 且不含 "net.neoforge"
    // 新版 Forge (1.20.1+) 的 library 拆分为 net.minecraftforge:fmlloader / JarJar* 等，
    // 没有 net.minecraftforge:forge: 这个 library，但 JSON 里仍有 "minecraftforge" 字样
    if json_lower.contains("minecraftforge") {
        return Some(VersionType::Forge);
    }

    // 兜底：解析 libraries 数组，防止极端情况下关键字未命中
    if let Some(libraries) = version_json["libraries"].as_array() {
        for lib in libraries {
            let name = lib["name"].as_str().unwrap_or("");
            // 老版 Forge：net.minecraftforge:forge:1.16.5-36.2.0
            if name.starts_with("net.minecraftforge:forge:") {
                return Some(VersionType::Forge);
            }
            if name.starts_with("net.neoforged:neoforge:") {
                return Some(VersionType::NeoForge);
            }
            if name.starts_with("net.fabricmc:fabric-loader:") {
                return Some(VersionType::Fabric);
            }
            if name.starts_with("org.quiltmc:quilt-loader:") {
                return Some(VersionType::Quilt);
            }
            if name.starts_with("com.mumfrey:liteloader:") {
                return Some(VersionType::LiteLoader);
            }
            if name.starts_with("optifine:OptiFine:") {
                return Some(VersionType::OptiFine);
            }
        }
    }
    None
}

/// 判断是否为远古版本（发布年份 2000-2013）
fn is_old_version(release_time: &str) -> bool {
    use chrono::Datelike;

    // 使用统一的时间解析工具
    if let Some(dt) = crate::utils::datetime::parse_utc(release_time) {
        let year = dt.year();
        return (2000..2013).contains(&year);
    }
    false
}

/// 判断是否为快照版本
fn is_snapshot(version_id: &str, json_type: &str) -> bool {
    if json_type == "snapshot" {
        return true;
    }
    let id_lower = version_id.to_lowercase();
    // 匹配 YYwWWa 格式（如 24w14a、23w13a）
    static RE_WEEKLY: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re_weekly = RE_WEEKLY.get_or_init(|| regex::Regex::new(r"^\d{2}w\d{2}[a-z]").unwrap());
    // 匹配 -pre\d+ 或 -rc\d+ 格式（如 1.20.1-pre1、1.20.1-rc1）
    static RE_PRE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re_pre = RE_PRE.get_or_init(|| regex::Regex::new(r"-pre\d+").unwrap());
    static RE_RC: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re_rc = RE_RC.get_or_init(|| regex::Regex::new(r"-rc\d+").unwrap());
    re_weekly.is_match(&id_lower)
        || re_pre.is_match(&id_lower)
        || re_rc.is_match(&id_lower)
        || id_lower.contains("experimental")
        || id_lower.contains("combat")
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
