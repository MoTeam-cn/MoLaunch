//! 库解析：从版本 JSON 解析库列表 + 规则匹配 + 去重

use std::collections::HashMap;
use std::path::Path;

use super::{maven_to_path, LibEntry};

/// Check if a native classifier matches the current platform architecture.
///
/// Mojang's version JSON for lwjgl 3.4.x has `natives-windows`, `natives-windows-x86`,
/// and `natives-windows-arm64` entries that all have identical rules (`os.name=windows`
/// with no `arch` field). Without this filter, all three pass `check_rules` and then
/// collide in `deduplicate_libs` (same key `group:artifact:true`), causing only the
/// last one in JSON order to survive — typically the wrong architecture.
pub fn is_native_matching_arch(classifier: &str) -> bool {
    if cfg!(target_os = "windows") {
        let is_64bit = std::mem::size_of::<usize>() == 8;
        match classifier {
            "natives-windows" => is_64bit,      // 64-bit x86
            "natives-windows-x86" => !is_64bit, // 32-bit x86
            "natives-windows-arm64" => false,   // ARM64 not detected via usize; skip for now
            _ => true,                          // Unknown windows native, allow
        }
    } else if cfg!(target_os = "macos") {
        let is_arm64 = cfg!(target_arch = "aarch64");
        match classifier {
            "natives-macos" => !is_arm64,
            "natives-macos-arm64" => is_arm64,
            _ => true,
        }
    } else {
        true // Linux and others: no arch-specific natives to filter
    }
}

/// Check if rules match current platform
pub fn check_rules(rules: &Option<Vec<serde_json::Value>>) -> bool {
    let rules = match rules {
        Some(r) => r,
        None => return true,
    };

    if rules.is_empty() {
        return true;
    }

    let mut required = false;

    for rule in rules {
        let action = rule["action"].as_str().unwrap_or("allow");
        let mut is_right_rule = true;

        if let Some(os) = rule.get("os") {
            if let Some(name) = os["name"].as_str() {
                if name != "windows" {
                    is_right_rule = false;
                }
            }
            if let Some(arch) = os["arch"].as_str() {
                let is_64bit = std::mem::size_of::<usize>() == 8;
                if (arch == "x86") == is_64bit {
                    is_right_rule = false;
                }
            }
        }

        if let Some(features) = rule.get("features") {
            if features.get("is_demo_user").is_some() {
                is_right_rule = false;
            }
        }

        if action == "allow" {
            if is_right_rule {
                required = true;
            }
        } else {
            if is_right_rule {
                required = false;
            }
        }
    }

    required
}

/// Parse all libraries from version JSON
pub fn parse_libraries(json: &serde_json::Value, game_dir: &Path) -> Vec<LibEntry> {
    let mut result = Vec::new();

    let libraries = match json["libraries"].as_array() {
        Some(libs) => libs,
        None => return result,
    };

    for library in libraries {
        let rules = library
            .get("rules")
            .and_then(|r| r.as_array())
            .map(|arr| arr.clone());
        if !check_rules(&rules) {
            continue;
        }

        let name = library["name"].as_str().unwrap_or_default();
        // 拼接根 URL：去掉结尾斜杠后用 "/" 连接相对路径
        // Fabric 格式: url="https://maven.fabricmc.net/" + path="org/ow2/asm/..."
        // 结果应为: "https://maven.fabricmc.net/org/ow2/asm/..."
        // 注意：必须用 "/" 连接，否则会拼成 "maven.fabricmc.netorg/ow2/asm/..."
        let root_url = library["url"].as_str().map(|u| {
            let path = crate::minecraft::utils::maven::maven_to_relative_path(name);
            format!("{}/{}", u.trim_end_matches('/'), path)
        });

        if let Some(natives) = library.get("natives") {
            if let Some(windows_name) = natives["windows"].as_str() {
                let arch = if std::mem::size_of::<usize>() == 8 {
                    "64"
                } else {
                    "32"
                };
                let classifier = windows_name.replace("${arch}", arch);

                if let Some(cls) = library["downloads"]["classifiers"].get(&classifier) {
                    let url = cls["url"]
                        .as_str()
                        .or(root_url.as_deref())
                        .map(|s| s.to_string());
                    let path = if let Some(p) = cls["path"].as_str() {
                        if p.contains("..") {
                            crate::log_warn!(
                                "[Libraries] Skip path traversal in artifact path: {}",
                                p
                            );
                            continue;
                        }
                        game_dir
                            .join("libraries")
                            .join(p.replace('/', std::path::MAIN_SEPARATOR_STR))
                            .to_string_lossy()
                            .to_string()
                    } else {
                        maven_to_path(name, game_dir)
                            .replace(".jar", &format!("-natives-{}.jar", arch))
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
                    let path = maven_to_path(name, game_dir)
                        .replace(".jar", &format!("-natives-{}.jar", arch));
                    result.push(LibEntry {
                        original_name: Some(name.to_string()),
                        local_path: path,
                        size: 0,
                        is_natives: true,
                        sha1: None,
                        url: root_url,
                    });
                }
            }
        } else if name.split(':').count() > 3
            && name.split(':').nth(3).unwrap_or("").starts_with("natives-")
        {
            // 新格式（Forge 26.2+）：无 "natives" 字段，但 name 含 natives-xxx classifier
            // 直接用 downloads.artifact 的路径和 URL
            let classifier = name.split(':').nth(3).unwrap_or("");

            // 架构过滤：Mojang 的 rules 只检查 os.name 不检查 arch
            // 三个 windows native 变体都会通过 check_rules，需要在这里过滤
            if !is_native_matching_arch(classifier) {
                continue;
            }

            let (url, local_path, size, sha1) = if let Some(artifact) =
                library.get("downloads").and_then(|d| d.get("artifact"))
            {
                let url = artifact["url"]
                    .as_str()
                    .or(root_url.as_deref())
                    .map(|s| s.to_string());
                let path = if let Some(p) = artifact["path"].as_str() {
                    if p.contains("..") {
                        crate::log_warn!("[Libraries] Skip path traversal in artifact path: {}", p);
                        continue;
                    }
                    game_dir
                        .join("libraries")
                        .join(p.replace('/', std::path::MAIN_SEPARATOR_STR))
                        .to_string_lossy()
                        .to_string()
                } else {
                    maven_to_path(name, game_dir)
                };
                let size = artifact["size"].as_i64().unwrap_or(0);
                let sha1 = artifact["sha1"].as_str().map(|s| s.to_string());
                (url, path, size, sha1)
            } else {
                // 没有 downloads.artifact：从根级别读取 size/sha1（Fabric 格式）
                let size = library["size"].as_i64().unwrap_or(0);
                let sha1 = library["sha1"].as_str().map(|s| s.to_string());
                (root_url, maven_to_path(name, game_dir), size, sha1)
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
            let (url, local_path, size, sha1) = if let Some(artifact) =
                library.get("downloads").and_then(|d| d.get("artifact"))
            {
                let url = artifact["url"]
                    .as_str()
                    .or(root_url.as_deref())
                    .map(|s| s.to_string());
                let path = if let Some(p) = artifact["path"].as_str() {
                    if p.contains("..") {
                        crate::log_warn!("[Libraries] Skip path traversal in artifact path: {}", p);
                        continue;
                    }
                    game_dir
                        .join("libraries")
                        .join(p.replace('/', std::path::MAIN_SEPARATOR_STR))
                        .to_string_lossy()
                        .to_string()
                } else {
                    maven_to_path(name, game_dir)
                };
                let size = artifact["size"].as_i64().unwrap_or(0);
                let sha1 = artifact["sha1"].as_str().map(|s| s.to_string());
                (url, path, size, sha1)
            } else {
                // 没有 downloads.artifact：从根级别读取 size/sha1（Fabric 格式）
                let size = library["size"].as_i64().unwrap_or(0);
                let sha1 = library["sha1"].as_str().map(|s| s.to_string());
                (root_url, maven_to_path(name, game_dir), size, sha1)
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
