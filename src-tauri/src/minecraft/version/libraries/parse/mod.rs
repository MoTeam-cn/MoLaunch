//! 库解析：从版本 JSON 解析库 + 规则推导 + 路径解析 + 去重

mod path;
mod rules;

use std::collections::HashMap;
use std::path::Path;

use super::LibEntry;
use path::{maven_natives_path, resolve_artifact};

pub use rules::{check_rules, is_native_matching_arch};

/// Parse all libraries from version JSON
pub fn parse_libraries(json: &serde_json::Value, game_dir: &Path) -> Vec<LibEntry> {
    let mut result = Vec::new();

    let libraries = match json["libraries"].as_array() {
        Some(libs) => libs,
        None => return result,
    };

    for library in libraries {
        let rules = library.get("rules").and_then(|r| r.as_array()).cloned();
        if !check_rules(&rules) {
            continue;
        }

        let name = library["name"].as_str().unwrap_or_default();
        // 拼接根 URL：去掉结尾斜杠后用 "/" 连接相对路径
        // Fabric 格式: url="https://maven.fabricmc.net/" + path="org/ow2/asm/..."
        // 注意：必须用 "/" 连接，否则会拼成 "maven.fabricmc.netorg/ow2/asm/..."
        let root_url = library["url"].as_str().map(|u| {
            let path = crate::minecraft::utils::maven::maven_to_relative_path(name);
            format!("{}/{}", u.trim_end_matches('/'), path)
        });
        let root_url = root_url.as_deref();

        if let Some(natives) = library.get("natives") {
            if let Some(windows_name) = natives["windows"].as_str() {
                let arch = if std::mem::size_of::<usize>() == 8 {
                    "64"
                } else {
                    "32"
                };
                let classifier = windows_name.replace("${arch}", arch);

                if let Some(cls) = library["downloads"]["classifiers"].get(&classifier) {
                    let url = cls["url"].as_str().or(root_url).map(|s| s.to_string());
                    let path = if let Some(p) = cls["path"].as_str() {
                        let Some(local) = path::artifact_local_path(p, game_dir) else {
                            continue;
                        };
                        local
                    } else {
                        maven_natives_path(name, game_dir, arch)
                    };
                    let size = cls["size"].as_i64().unwrap_or(0);
                    let sha1 = cls["sha1"].as_str().map(|s| s.to_string());

                    result.push(LibEntry {
                        original_name: Some(name.to_string()),
                        local_path: path,
                        size,
                        is_natives: true,
                        sha1,
                        url,
                    });
                } else {
                    let path = maven_natives_path(name, game_dir, arch);
                    result.push(LibEntry {
                        original_name: Some(name.to_string()),
                        local_path: path,
                        size: 0,
                        is_natives: true,
                        sha1: None,
                        url: root_url.map(|s| s.to_string()),
                    });
                }
            }
        } else if name.split(':').count() > 3
            && name.split(':').nth(3).unwrap_or("").starts_with("natives-")
        {
            // 新格式（Forge 26.2+）：无 "natives" 字段，但 name 含 natives-xxx classifier
            let classifier = name.split(':').nth(3).unwrap_or("");

            // 架构过滤：Mojang 的 rules 只检查 os.name 不检查 arch，
            // 三个 windows native 变体都会通过 check_rules，需要在这里过滤
            if !is_native_matching_arch(classifier) {
                continue;
            }

            let Some((url, local_path, size, sha1)) =
                resolve_artifact(library, name, game_dir, root_url)
            else {
                continue;
            };

            result.push(LibEntry {
                original_name: Some(name.to_string()),
                local_path,
                size,
                is_natives: true,
                sha1,
                url,
            });
        } else {
            let Some((url, local_path, size, sha1)) =
                resolve_artifact(library, name, game_dir, root_url)
            else {
                continue;
            };

            result.push(LibEntry {
                original_name: Some(name.to_string()),
                local_path,
                size,
                is_natives: false,
                sha1,
                url,
            });
        }
    }

    deduplicate_libs(result)
}

/// Deduplicate libraries (keep newer version)
fn deduplicate_libs(libs: Vec<LibEntry>) -> Vec<LibEntry> {
    let mut map: HashMap<String, LibEntry> = HashMap::new();

    for lib in libs {
        let key = format!("{}:{}", lib.name(), lib.is_natives);

        if let Some(existing) = map.get(&key) {
            let existing_version =
                get_version_from_name(existing.original_name.as_deref().unwrap_or(""));
            let new_version = get_version_from_name(lib.original_name.as_deref().unwrap_or(""));
            if compare_versions_ge(&new_version, &existing_version) {
                map.insert(key, lib);
            }
        } else {
            map.insert(key, lib);
        }
    }

    map.into_values().collect()
}

fn get_version_from_name(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() >= 3 {
        parts[2].to_string()
    } else {
        String::new()
    }
}

fn compare_versions_ge(a: &str, b: &str) -> bool {
    let a_parts = crate::utils::version::parse_number(a);
    let b_parts = crate::utils::version::parse_number(b);

    let max_len = a_parts.len().max(b_parts.len());
    for i in 0..max_len {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);
        if a_val > b_val {
            return true;
        } else if a_val < b_val {
            return false;
        }
    }
    true
}