//! options.txt 资源包操作公共抽象（resourcePacks / shaderPack）
//! 离线皮肤与资源包/光影管理共用。

use std::path::Path;

use crate::utils::version::parse_number as parse_version_number;

/// MC 1.13+ 对 zip 资源包使用 `file/` 前缀
pub fn use_file_prefix(mc_version: &str) -> bool {
    let v = parse_version_number(mc_version);
    let major = v.first().copied().unwrap_or(0);
    let minor = v.get(1).copied().unwrap_or(0);
    major > 1 || (major == 1 && minor >= 13)
}

/// 资源包引用名：文件夹直接目录名，zip 按 MC 版本决定是否带 file/ 前缀
pub fn pack_ref(pack_name: &str, is_folder: bool, mc_version: &str) -> String {
    if is_folder {
        pack_name.to_string()
    } else if use_file_prefix(mc_version) {
        format!("file/{}", pack_name)
    } else {
        pack_name.to_string()
    }
}

fn read_options_lines(game_dir: &Path) -> Vec<String> {
    let path = game_dir.join("options.txt");
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .map(|c| c.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default()
}

fn write_options_lines(game_dir: &Path, lines: &[String]) {
    let _ = std::fs::write(game_dir.join("options.txt"), lines.join("\n"));
}

/// 解析 resourcePacks 行，返回数组内容（去首尾括号）
fn parse_resource_packs_content(trimmed: &str) -> Option<String> {
    if !trimmed.starts_with("resourcePacks:") {
        return None;
    }
    let start = trimmed.find('[')?;
    let end = trimmed.rfind(']')?;
    if end <= start {
        return None;
    }
    Some(trimmed[start + 1..end].to_string())
}

/// 启用/停用资源包（写 options.txt 的 resourcePacks 数组）
pub fn set_resource_pack_enabled(
    game_dir: &Path,
    pack_name: &str,
    is_folder: bool,
    enabled: bool,
    mc_version: &str,
) -> Result<(), String> {
    let ref_name = pack_ref(pack_name, is_folder, mc_version);
    let desired = format!("\"{}\"", ref_name);
    let mut lines = read_options_lines(game_dir);
    let mut replaced = false;
    for line in lines.iter_mut() {
        if let Some(content) = parse_resource_packs_content(line.trim()) {
            let mut entries: Vec<String> = content
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            // 按去引号后的引用名匹配（兼容旧版裸名 / 1.13+ file/ 前缀）
            let entry_matches = |e: &String| {
                let unquoted = e.trim().trim_matches('"');
                unquoted == pack_name || unquoted == ref_name
            };
            entries.retain(|e| !entry_matches(e));
            if enabled {
                entries.push(desired.clone());
            }
            let prefix_end = line.find('[').unwrap_or(line.len());
            *line = format!("{}[{}]", &line[..prefix_end], entries.join(","));
            replaced = true;
            break;
        }
    }
    if !replaced && enabled {
        lines.push(format!("resourcePacks:[{}]", desired));
    }
    write_options_lines(game_dir, &lines);
    Ok(())
}

/// 设置/清除光影（shaderPack 键；None 清除）
pub fn set_shader_pack(game_dir: &Path, pack_name: Option<&str>) -> Result<(), String> {
    let desired = match pack_name {
        Some(name) => format!("shaderPack:\"{}\"", name),
        None => "shaderPack:\"\"".to_string(),
    };
    let mut lines = read_options_lines(game_dir);
    let mut replaced = false;
    for line in lines.iter_mut() {
        if line.trim().starts_with("shaderPack:") {
            *line = desired.clone();
            replaced = true;
            break;
        }
    }
    if !replaced {
        lines.push(desired);
    }
    write_options_lines(game_dir, &lines);
    Ok(())
}
