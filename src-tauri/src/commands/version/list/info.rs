//! 版本元信息查询（有效目录 / 游戏版本号 / 加载器信息）

use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::version::modpack_meta::ModpackMetaFile;
use crate::minecraft::version::scan as version_scan;
use crate::minecraft::version::setup::VersionSetup;
use crate::minecraft::version::state::VersionType;
use crate::state::AppState;

use super::super::sanitize_version_id;
use super::{detect_version_type_from_dir, resolve_isolation_mode, version_type_to_string};

/// 获取版本的有效游戏目录（考虑版本隔离）
///
/// 隔离时返回 `{game_dir}/versions/{version_id}/`
/// 非隔离时返回 `{game_dir}/`
pub async fn get_version_effective_dir(
    state: &AppState,
    version_id: String,
) -> Result<String, String> {
    sanitize_version_id(&version_id)?;

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let global_isolation_mode = state.config.lock().await.isolation_mode;

    // 版本独立隔离设置覆盖全局
    let isolation_mode = resolve_isolation_mode(&game_dir, &version_id, global_isolation_mode);
    let version_type = detect_version_type_from_dir(&game_dir, &version_id);
    let mode = IsolationMode::from_u32(isolation_mode);
    let effective_dir =
        isolation::get_effective_game_dir(&game_dir, &version_id, mode, version_type);

    Ok(effective_dir.to_string_lossy().to_string())
}

/// 获取版本对应的 Minecraft 游戏版本号（如 "1.20.1"）
///
/// 用于从 ModTab 打开资源详情弹窗时，自动选中整合包对应的版本筛选 tag。
/// 解析顺序参考 `version::scan::extract_original_version`：
/// 1. JSON 的 `inheritsFrom` 字段
/// 2. arguments.game 中的 `--fml.mcVersion`
/// 3. downloads.client.url 正则匹配
/// 4. JSON 的 `jar` 字段
/// 5. JSON 的 `id` 字段正则匹配
///
/// JSON 缺失或以上策略均无法提取时，回退读取 `modpack.meta.json` 的 `mc_version`
///（整合包安装时写入的权威版本），两者皆无时返回 `None`。
pub async fn get_version_game_version(
    state: &AppState,
    version_id: String,
) -> Result<Option<String>, String> {
    sanitize_version_id(&version_id)?;

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;

    let version_dir = game_dir.join("versions").join(&version_id);
    let json_path = version_dir.join(format!("{}.json", version_id));
    if json_path.exists() {
        let content = std::fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read version JSON: {}", e))?;
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse version JSON: {}", e))?;
        if let Some(mc) = version_scan::extract_original_version(&json, &content) {
            return Ok(Some(mc));
        }
    }

    // 版本 JSON 缺失，或提取不到 MC 版本时（旧版整合包 JSON 的 id 为非数字名、无 inheritsFrom），
    // 回退 setup.ini 的 OriginalVersion（所有安装方式通用）；再回退 modpack.meta.json（整合包安装写入的权威）
    if let Ok(Some(setup)) = VersionSetup::load(&version_dir) {
        let v = &setup.loader.original_version;
        if !v.is_empty() && v != "unknown" {
            return Ok(Some(v.clone()));
        }
    }
    Ok(ModpackMetaFile::load(&version_dir)
        .map_err(|e| format!("Failed to read modpack.meta.json: {}", e))?
        .filter(|m| !m.mc_version.is_empty())
        .map(|m| m.mc_version))
}

/// 获取版本加载器信息（加载器类型 + 加载器版本号）
///
/// 读取 `versions/{id}/setup.ini` 的 `Type` 字段和对应的 `XxxVersion` 字段，
/// 用于创建联机房间时上报 `host_loader` / `host_loader_version`。
///
/// 返回 `(loader_type, loader_version)`：
/// - `loader_type`：`forge` / `fabric` / `neoforge` / `quilt` / `optifine` / `liteloader` / `release` / `snapshot` / `old` / `unknown`
/// - `loader_version`：对应加载器的版本号（如 `47.2.0`），无加载器时为空字符串
///
/// setup.ini 缺失或缺少加载器版本时，`load_or_create` 会从版本 JSON 的 libraries 回填并持久化；
/// 仍无法给出版本时再回退 `modpack.meta.json`（在线整合包更准确），再仍缺则返回空版本号。
pub async fn get_version_loader_info(
    state: &AppState,
    version_id: String,
) -> Result<(String, String), String> {
    sanitize_version_id(&version_id)?;

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let version_dir = game_dir.join("versions").join(&version_id);

    // 优先读 setup.ini（load_or_create 会对缺失的加载器版本做 JSON 回填并持久化）
    let setup = VersionSetup::load_or_create(&version_dir, &version_id);
    let loader_type = version_type_to_string(&setup.loader.version_type);
    let loader_version = match setup.loader.version_type {
        VersionType::Forge => setup.loader.forge_version.clone().unwrap_or_default(),
        VersionType::NeoForge => setup.loader.neoforge_version.clone().unwrap_or_default(),
        VersionType::Fabric => setup.loader.fabric_version.clone().unwrap_or_default(),
        VersionType::Quilt => setup.loader.quilt_version.clone().unwrap_or_default(),
        VersionType::OptiFine => setup.loader.optifine_version.clone().unwrap_or_default(),
        VersionType::LiteLoader => setup.loader.liteloader_version.clone().unwrap_or_default(),
        _ => String::new(),
    };

    // 保底：setup 无法给出加载器版本时，回退整合包元数据 modpack.meta.json（在线整合包更准确）
    if loader_version.is_empty() {
        if let Ok(Some(meta)) = ModpackMetaFile::load(&version_dir) {
            if let Some(lt) = meta.loader {
                return Ok((lt, meta.loader_version.unwrap_or_default()));
            }
        }
    }
    Ok((loader_type, loader_version))
}
