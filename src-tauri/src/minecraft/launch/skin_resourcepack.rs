//! 离线账号皮肤资源包生成模块（PCL2 方案 B）
//!
//! 通过生成资源包 zip 替换原版玩家纹理，让离线账号的自定义皮肤在游戏中生效。
//! 与 `adjust_uuid_for_skin_variant`（方案 A）互补：
//! - 方案 A 调整 UUID 保证模型类型（classic/slim）正确
//! - 方案 B 用资源包替换所有角色纹理为用户选定的皮肤，确保 1.19.3+ 也精确显示
//!
//! 资源包结构：
//! ```text
//! MoLaunch Skin.zip
//! ├── pack.mcmeta          (pack_format 按版本选择)
//! ├── pack.png             (资源包图标，用皮肤文件本身)
//! └── assets/minecraft/textures/entity/
//!     ├── alex.png         (1.19.3 以前)
//!     ├── steve.png        (1.19.3 以前)
//!     └── player/          (1.19.3+)
//!         ├── slim/
//!         │   ├── alex.png
//!         │   ├── ari.png
//!         │   └── ... (9 个角色)
//!         └── wide/
//!             ├── steve.png
//!             ├── kai.png
//!             └── ... (9 个角色)
//! ```

use crate::log_info;
use crate::minecraft::loaders::utils::parse_version_number;
use std::io::Write;
use std::path::Path;

/// 资源包文件名（固定，用于 options.txt 中的引用）
const SKIN_PACK_NAME: &str = "MoLaunch Skin.zip";

/// 1.19.3+ 的 9 个默认角色名
const DEFAULT_SKINS_1193: &[&str] = &[
    "alex", "ari", "efe", "kai", "makena", "noor", "steve", "sunny", "zuri",
];

/// 根据MC版本返回对应的 pack_format
///
/// 参考 PCL2 的版本映射，覆盖 1.6 到 1.20.5+
fn get_pack_format(mc_version: &str) -> u32 {
    let v = parse_version_number(mc_version);
    let major = v.first().copied().unwrap_or(0);
    let minor = v.get(1).copied().unwrap_or(0);
    let patch = v.get(2).copied().unwrap_or(0);

    // 参考 https://minecraft.wiki/w/Pack_format
    match (major, minor, patch) {
        (1, 6..=7, _) => 1,
        (1, 8..=8, _) => 1,
        (1, 9..=9, _) => 2,
        (1, 10..=10, _) => 2,
        (1, 11..=11, _) => 3,
        (1, 12..=12, _) => 4,
        (1, 13..=13, _) => 5,
        (1, 14..=14, _) => 6,
        (1, 15..=15, _) => 6,
        (1, 16, 0..=1) => 6,
        (1, 16, 2..=5) => 7,
        (1, 17..=17, _) => 8,
        (1, 18..=18, _) => 9,
        (1, 19, 0..=2) => 9,
        (1, 19, 3) => 12,
        (1, 19, 4) => 13,
        (1, 19, 5..=u32::MAX) => 15,
        (1, 20, 0..=1) => 15,
        (1, 20, 2) => 18,
        (1, 20, 3..=4) => 22,
        (1, 20, 5..=u32::MAX) => 34,
        (1, 21..=u32::MAX, _) => 34,
        _ => 15, // 默认用较新的格式
    }
}

/// 判断是否为 1.19.3+ 版本（角色纹理路径不同）
fn is_version_1193_plus(mc_version: &str) -> bool {
    let v = parse_version_number(mc_version);
    let major = v.first().copied().unwrap_or(0);
    let minor = v.get(1).copied().unwrap_or(0);
    let patch = v.get(2).copied().unwrap_or(0);
    major > 1 || (major == 1 && (minor > 19 || (minor == 19 && patch >= 3)))
}

/// 生成资源包中需要替换的纹理文件路径列表
///
/// 1.19.3+ 替换 slim/wide 目录下全部 9 个角色；
/// 1.19.3 以前只替换 alex.png / steve.png。
fn get_texture_paths(mc_version: &str, slim: bool) -> Vec<String> {
    if is_version_1193_plus(mc_version) {
        let model_dir = if slim { "slim" } else { "wide" };
        DEFAULT_SKINS_1193
            .iter()
            .map(|name| {
                format!(
                    "assets/minecraft/textures/entity/player/{}/{}.png",
                    model_dir, name
                )
            })
            .collect()
    } else {
        let filename = if slim { "alex.png" } else { "steve.png" };
        vec![format!("assets/minecraft/textures/entity/{}", filename)]
    }
}

/// 生成 pack.mcmeta 内容
fn build_pack_mcmeta(pack_format: u32) -> String {
    format!(
        r#"{{"pack":{{"pack_format":{},"description":"MoLaunch offline skin"}}}}"#,
        pack_format
    )
}

/// 生成资源包 zip 文件
///
/// 将皮肤 PNG 文件写入 zip 中所有需要替换的纹理路径，
/// 同时写入 pack.mcmeta 和 pack.png。
fn create_skin_pack(
    zip_path: &Path,
    skin_png: &[u8],
    texture_paths: &[String],
    pack_format: u32,
) -> anyhow::Result<()> {
    let file = std::fs::File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // pack.mcmeta
    zip.start_file("pack.mcmeta", options)?;
    zip.write_all(build_pack_mcmeta(pack_format).as_bytes())?;

    // pack.png（用皮肤文件本身作为图标）
    zip.start_file("pack.png", options)?;
    zip.write_all(skin_png)?;

    // 皮肤纹理文件（替换所有角色路径）
    for tex_path in texture_paths {
        zip.start_file(tex_path, options)?;
        zip.write_all(skin_png)?;
    }

    zip.finish()?;
    Ok(())
}

/// 在 options.txt 中启用资源包
///
/// MC 1.13+ 使用 `resourcePacks:["file/MoLaunch Skin.zip"]` 格式，
/// 1.6-1.12 使用 `resourcePacks:["MoLaunch Skin.zip"]` 格式。
fn enable_resourcepack_in_options(game_dir: &Path, mc_version: &str) {
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
                    *line = format!("{}[{}]{}", &line[..line.find('[').unwrap_or(0)], desired_entry, after);
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
fn disable_resourcepack_in_options(game_dir: &Path) {
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
/// 1. 释放嵌入的皮肤 PNG 到临时变量
/// 2. 生成资源包 zip 到 `<game_dir>/resourcepacks/MoLaunch Skin.zip`
/// 3. 修改 options.txt 启用资源包
///
/// 如果 skin_name 为 None 或不在默认皮肤列表中，则移除已有资源包。
pub fn apply_skin_resourcepack(
    game_dir: &Path,
    mc_version: &str,
    skin_name: Option<&str>,
) -> anyhow::Result<()> {
    let pack_dir = game_dir.join("resourcepacks");
    let zip_path = pack_dir.join(SKIN_PACK_NAME);

    match skin_name {
        Some(name) => {
            // 获取嵌入的皮肤 PNG
            let resource_path = format!("skins/{}.png", name);
            let skin_png = crate::resources::get_embedded_resource(&resource_path)
                .ok_or_else(|| anyhow::anyhow!("Skin PNG not found: {}", resource_path))?;

            // 判断变体
            let slim = matches!(
                name,
                "Alex" | "Ari" | "Efe" | "Makena" | "Noor" | "Sunny" | "Zuri"
            );

            let pack_format = get_pack_format(mc_version);
            let texture_paths = get_texture_paths(mc_version, slim);

            // 确保目录存在
            std::fs::create_dir_all(&pack_dir)?;

            // 生成资源包
            create_skin_pack(&zip_path, skin_png, &texture_paths, pack_format)?;

            // 启用资源包
            enable_resourcepack_in_options(game_dir, mc_version);

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
            // 移除已有资源包
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
    disable_resourcepack_in_options(game_dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_format() {
        assert_eq!(get_pack_format("1.12.2"), 4);
        assert_eq!(get_pack_format("1.16.5"), 7);
        assert_eq!(get_pack_format("1.19.2"), 9);
        assert_eq!(get_pack_format("1.19.3"), 12);
        assert_eq!(get_pack_format("1.20.1"), 15);
    }

    #[test]
    fn test_is_1193_plus() {
        assert!(!is_version_1193_plus("1.19.2"));
        assert!(is_version_1193_plus("1.19.3"));
        assert!(is_version_1193_plus("1.20.1"));
    }

    #[test]
    fn test_texture_paths_1192() {
        let paths = get_texture_paths("1.19.2", true);
        assert_eq!(paths.len(), 1);
        assert!(paths[0].contains("alex.png"));
    }

    #[test]
    fn test_texture_paths_1193() {
        let paths = get_texture_paths("1.19.3", false);
        assert_eq!(paths.len(), 9);
        // DEFAULT_SKINS_1193 按字母序排列，第一个是 alex
        assert!(paths[0].contains("player/wide/alex.png"));
        assert!(paths[6].contains("player/wide/steve.png"));
    }
}
