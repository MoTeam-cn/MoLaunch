//! 资源包安装与移除
//!
//! 生成 zip 后写入 options.txt 启用/停用资源包。

use crate::log_info;
use crate::utils::version::parse_number as parse_version_number;
use std::path::Path;

use super::generate::{create_skin_pack, get_pack_format, get_texture_paths};

/// 资源包文件名（固定，用于 options.txt 中的引用）
const SKIN_PACK_NAME: &str = "MoLaunch Skin.zip";

/// 在 options.txt 中启用资源包
///
/// MC 1.13+ 使用 `resourcePacks:["file/MoLaunch Skin.zip"]` 格式，
/// 1.6-1.12 使用 `resourcePacks:["MoLaunch Skin.zip"]` 格式。
fn enable_resource_pack_in_options(game_dir: &Path, mc_version: &str) {
    let options_path = game_dir.join("options.txt");
    let v = parse_version_number(mc_version);
    let major = v.first().copied().unwrap_or(0);
    let minor = v.get(1).copied().unwrap_or(0);
    let use_file_prefix = major > 1 || (major == 1 && minor >= 13);

    let pack_ref = if use_file_prefix {
        format!("file/{}", SKIN_PACK_NAME)
    } else {
        SKIN_PACK_NAME.to_string()
    };

    let desired_entry = format!("\"{}\"", pack_ref);
    let desired_line = format!("resourcePacks:[{}]", desired_entry);

    if !options_path.exists() {
        let _ = std::fs::write(&options_path, &desired_line);
        return;
    }

    let content = std::fs::read_to_string(&options_path).unwrap_or_default();
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    let mut found = false;
    for line in lines.iter_mut() {
        let trimmed = line.trim();
        if trimmed.starts_with("resourcePacks:") {
            if trimmed.contains(SKIN_PACK_NAME) {
                // 已包含我们的资源包，无需修改
                found = true;
                break;
            }
            // 在已有的数组中添加我们的资源包
            // 格式: resourcePacks:["xxx","yyy"] → resourcePacks:["xxx","yyy","file/MoLaunch Skin.zip"]
            if let Some(close) = trimmed.rfind(']') {
                let insert_pos = line.len() - (trimmed.len() - close);
                let before = &line[..insert_pos];
                let after = &line[insert_pos..];
                // 判断数组是否为空
                let array_content = &trimmed[trimmed.find('[').unwrap_or(0) + 1..close];
                if array_content.trim().is_empty() {
                    *line = format!(
                        "{}[{}]{}",
                        &line[..line.find('[').unwrap_or(0)],
                        desired_entry,
                        after
                    );
                } else {
                    *line = format!("{},{}{}", before, desired_entry, after);
                }
                found = true;
                break;
            }
        }
    }

    if !found {
        lines.push(desired_line);
    }

    let new_content = lines.join("\n");
    let _ = std::fs::write(&options_path, new_content);
}

/// 从 options.txt 中移除资源包
fn disable_resource_pack_in_options(game_dir: &Path) {
    let options_path = game_dir.join("options.txt");
    if !options_path.exists() {
        return;
    }

    let content = std::fs::read_to_string(&options_path).unwrap_or_default();
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    for line in lines.iter_mut() {
        let trimmed = line.trim();
        if trimmed.starts_with("resourcePacks:") && trimmed.contains(SKIN_PACK_NAME) {
            // 移除我们的资源包引用
            // 简化处理：如果数组中只有我们的资源包，改为空数组
            let cleaned = trimmed.replace(&format!("\"file/{}\"", SKIN_PACK_NAME), "");
            let cleaned = cleaned.replace(&format!("\"{}\"", SKIN_PACK_NAME), "");
            // 清理多余的逗号
            let cleaned = cleaned.replace("[,", "[").replace(",]", "]");
            *line = cleaned;
            break;
        }
    }

    let new_content = lines.join("\n");
    let _ = std::fs::write(&options_path, new_content);
}

/// 应用离线皮肤资源包
///
/// 1. 获取皮肤 PNG（嵌入的默认皮肤或自定义文件）
/// 2. 生成资源包 zip 到 `<game_dir>/resourcepacks/MoLaunch Skin.zip`
/// 3. 修改 options.txt 启用资源包
///
/// skin_name 格式：
/// - 默认皮肤：`"Steve"` / `"Alex"` 等（从嵌入资源读取）
/// - 自定义皮肤：`"custom:/path/to/skin.png"`（从本地文件读取）
/// - None：移除已有资源包
pub fn apply_skin_resourcepack(
    game_dir: &Path,
    mc_version: &str,
    skin_name: Option<&str>,
) -> anyhow::Result<()> {
    let pack_dir = game_dir.join("resourcepacks");
    let zip_path = pack_dir.join(SKIN_PACK_NAME);

    match skin_name {
        Some(name) if name.starts_with("custom:") => {
            // 自定义皮肤：从本地文件读取 PNG
            let custom_path = &name["custom:".len()..];
            let skin_png = std::fs::read(custom_path)
                .map_err(|e| anyhow::anyhow!("读取自定义皮肤文件失败 {}: {}", custom_path, e))?;

            // 验证 PNG 文件头
            if skin_png.len() < 8 || skin_png[0..5] != [0x89, 0x50, 0x4E, 0x47, 0x0D] {
                anyhow::bail!("自定义皮肤文件不是有效的 PNG: {}", custom_path);
            }

            let pack_format = get_pack_format(mc_version);
            // 自定义皮肤的变体无法从文件名判断，默认用 classic（Steve 模型）
            // 前端上传时可通过 skin_name 传递变体信息：custom:/path|slim
            let slim = name.contains("|slim");
            let texture_paths = get_texture_paths(mc_version, slim);

            std::fs::create_dir_all(&pack_dir)?;
            create_skin_pack(&zip_path, &skin_png, &texture_paths, pack_format)?;
            enable_resource_pack_in_options(game_dir, mc_version);

            log_info!(
                "[Skin] 生成自定义离线皮肤资源包: {} (file={}, slim={}, pack_format={})",
                zip_path.display(),
                custom_path,
                slim,
                pack_format
            );
        }
        Some(name) => {
            // 默认皮肤：从嵌入资源读取 PNG
            let resource_path = format!("skins/{}.png", name);
            let skin_png = crate::resources::get_embedded_resource(&resource_path)
                .ok_or_else(|| anyhow::anyhow!("Skin PNG not found: {}", resource_path))?;

            let slim = matches!(
                name,
                "Alex" | "Ari" | "Efe" | "Makena" | "Noor" | "Sunny" | "Zuri"
            );

            let pack_format = get_pack_format(mc_version);
            let texture_paths = get_texture_paths(mc_version, slim);

            std::fs::create_dir_all(&pack_dir)?;
            create_skin_pack(&zip_path, skin_png, &texture_paths, pack_format)?;
            enable_resource_pack_in_options(game_dir, mc_version);

            log_info!(
                "[Skin] 生成离线皮肤资源包: {} (skin={}, slim={}, pack_format={}, textures={})",
                zip_path.display(),
                name,
                slim,
                pack_format,
                texture_paths.len()
            );
        }
        None => {
            remove_skin_resourcepack(game_dir);
        }
    }

    Ok(())
}

/// 移除离线皮肤资源包
pub fn remove_skin_resourcepack(game_dir: &Path) {
    let zip_path = game_dir.join("resourcepacks").join(SKIN_PACK_NAME);
    if zip_path.exists() {
        let _ = std::fs::remove_file(&zip_path);
        log_info!("[Skin] 移除离线皮肤资源包: {}", zip_path.display());
    }
    disable_resource_pack_in_options(game_dir);
}