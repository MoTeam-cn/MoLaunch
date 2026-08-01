//! 加载器类型检测（OptiFine / Fabric / NeoForge / Forge / LiteLoader / Snapshot）

use super::super::state::VersionType;

/// 加载器检测结果元组（version_type / forge / neoforge / fabric / optifine / liteloader / extra）
type LoaderDetectResult = (
    VersionType,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

/// 检测加载器类型
pub(super) fn detect_loaders(json: &serde_json::Value, json_content: &str) -> LoaderDetectResult {
    let mut state = VersionType::Release;
    let mut forge_version = None;
    let mut neoforge_version = None;
    let mut fabric_version = None;
    let mut optifine_version = None;
    let mut liteloader_version = None;

    // 检测 OptiFine
    if json_content.contains("optifine") || json_content.contains("OptiFine") {
        state = VersionType::OptiFine;
        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if name.contains("optifine") || name.contains("OptiFine") {
                        optifine_version = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }

    // 检测 Fabric
    if json_content.contains("net.fabricmc:fabric-loader")
        || json_content.contains("org.quiltmc:quilt-loader")
    {
        state = VersionType::Fabric;
        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if name.contains("fabric-loader") {
                        fabric_version = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }

    // 检测 NeoForge
    if json_content.contains("net.neoforge") {
        state = VersionType::NeoForge;
        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if name.contains("neoforge") {
                        neoforge_version = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }

    // 检测 Forge（排除 NeoForge）
    if json_content.contains("minecraftforge") && !json_content.contains("net.neoforge") {
        state = VersionType::Forge;
        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if name.contains("minecraftforge") {
                        forge_version = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }

    // 检测 LiteLoader
    if json_content.contains("liteloader") {
        state = VersionType::LiteLoader;
        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if name.contains("liteloader") {
                        liteloader_version = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }

    // 获取原版版本号
    let original_version = super::version_extract::extract_original_version(json, json_content);

    // 判断是否为快照版
    if let Some(id) = json["id"].as_str() {
        if id.contains("snapshot") || id.contains("pre") || id.contains("rc") {
            state = VersionType::Snapshot;
        }
    }

    (
        state,
        original_version,
        forge_version,
        neoforge_version,
        fabric_version,
        optifine_version,
        liteloader_version,
    )
}
