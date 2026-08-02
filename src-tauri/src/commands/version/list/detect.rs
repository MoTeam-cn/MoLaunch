//! 版本类型检测与隔离模式解析（detect_version_type_from_dir / resolve_isolation_mode）

use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;

/// 根据版本独立隔离设置和全局设置，解析最终使用的 isolation_mode
///
/// - indie_type=0 或 None：跟随全局（返回 global_mode）
/// - indie_type=1：强制开启隔离（返回 4=IsolationAll）
/// - indie_type=2：强制关闭隔离（返回 0=IsolationNone）
pub fn resolve_isolation_mode(
    game_dir: &std::path::Path,
    version_id: &str,
    global_mode: u32,
) -> u32 {
    let version_dir = game_dir.join("versions").join(version_id);
    let setup = VersionSetup::load_or_create(&version_dir, version_id);
    match setup.display.indie_type.unwrap_or(0) {
        1 => 4,           // 强制隔离 → IsolationAll
        2 => 0,           // 强制不隔离 → IsolationNone
        _ => global_mode, // 跟随全局
    }
}

/// Detect version type from directory
pub fn detect_version_type_from_dir(game_dir: &std::path::Path, version_id: &str) -> VersionType {
    let version_dir = game_dir.join("versions").join(version_id);

    // 1. 优先从 JSON 检测（检查libraries中的加载器）
    let json_path = version_dir.join(format!("{}.json", version_id));
    if json_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let detected = VersionType::detect_from_json(version_id, &json);
                // 如果检测到加载器类型，直接返回
                if detected != VersionType::Release {
                    return detected;
                }
            }
        }
    }

    // 2. 从 setup.ini 读取（仅当JSON检测为Release时）
    //    复用 VersionSetup::load 解析 Type= 到 loader.version_type
    if let Ok(Some(setup)) = VersionSetup::load(&version_dir) {
        match setup.loader.version_type {
            // release / unknown 视为无有效加载器信息，继续后续检测
            VersionType::Release | VersionType::Unknown => {}
            t => return t,
        }
    }

    // 3. 从版本ID推断
    let id_lower = version_id.to_lowercase();
    if id_lower.contains("forge") {
        return VersionType::Forge;
    }
    if id_lower.contains("neoforge") {
        return VersionType::NeoForge;
    }
    if id_lower.contains("fabric") {
        return VersionType::Fabric;
    }
    if id_lower.contains("optifine") {
        return VersionType::OptiFine;
    }

    VersionType::Release
}
