//! 规则推导
//!
//! 依赖的 os/arch/feature 规则匹配与 natives 架构过滤。

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