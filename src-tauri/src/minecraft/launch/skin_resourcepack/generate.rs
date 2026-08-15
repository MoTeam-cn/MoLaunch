//! 资源包生成
//!
//! 按 MC 版本计算 pack_format、纹理路径，并将皮肤 PNG 写入 zip。

use crate::utils::version::parse_number as parse_version_number;
use std::io::Write;
use std::path::Path;

/// 1.19.3+ 的 9 个默认角色名
const DEFAULT_SKINS_1193: &[&str] = &[
    "alex", "ari", "efe", "kai", "makena", "noor", "steve", "sunny", "zuri",
];

/// 根据MC版本返回对应的 pack_format
///
/// 版本映射覆盖 1.6 到 1.20.5+；
/// crate 级可见，供资源包编辑器 pack_format 联动校验复用。
pub(crate) fn get_pack_format(mc_version: &str) -> u32 {
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
pub(super) fn get_texture_paths(mc_version: &str, slim: bool) -> Vec<String> {
    let texture_paths_119_plus = is_version_1193_plus(mc_version);
    if texture_paths_119_plus {
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
pub(super) fn create_skin_pack(
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

#[cfg(test)]
#[path = "../skin_resourcepack_tests.rs"]
mod tests;
