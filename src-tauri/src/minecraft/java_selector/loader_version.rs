//! 从版本 JSON 解析加载器版本号
//!
//! 加载器版本藏在 libraries 的 name 字段里（group:artifact:version），
//! 例如 net.fabricmc:fabric-loader:0.15.11。这里只做纯解析，不读文件。

use serde_json::Value;

/// 加载器 → libraries 中的 group:artifact 前缀
const LOADER_ARTIFACTS: [(&str, &str); 4] = [
    ("fabric", "net.fabricmc:fabric-loader"),
    ("quilt", "org.quiltmc:quilt-loader"),
    ("forge", "net.minecraftforge:forge"),
    ("neoforge", "net.neoforged:neoforge"),
];

/// 解析指定加载器的版本号；找不到返回 None
pub fn parse_loader_version(json: &Value, loader: &str) -> Option<String> {
    let loader_lower = loader.to_lowercase();
    let prefix = LOADER_ARTIFACTS
        .iter()
        .find(|(kind, _)| *kind == loader_lower)
        .map(|(_, artifact)| *artifact)?;

    let libraries = json.get("libraries")?.as_array()?;
    for library in libraries {
        let Some(name) = library.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        let mut parts = name.split(':');
        let (Some(group), Some(artifact), Some(version)) =
            (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if format!("{}:{}", group, artifact) == prefix {
            return Some(version.to_string());
        }
    }
    None
}

/// 解析 major.minor.patch 三元组；带后缀（如 0.15.11-beta.1）时取数字部分
pub fn parse_semver3(version: &str) -> Option<(u32, u32, u32)> {
    let mut numbers = [0u32; 3];
    let mut count = 0;
    for segment in version.split('.') {
        if count >= 3 {
            break;
        }
        // 只取开头的连续数字（11-beta 取 11）
        let digits: String = segment.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return None;
        }
        numbers[count] = digits.parse().ok()?;
        count += 1;
    }
    if count < 3 {
        return None;
    }
    Some((numbers[0], numbers[1], numbers[2]))
}

#[cfg(test)]
#[path = "loader_version_test.rs"]
mod loader_version_test;
