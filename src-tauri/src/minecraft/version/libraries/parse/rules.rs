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
    check_rules_with_features(rules, &[])
}

/// Check if rules match current platform and feature flags.
///
/// 高版本 version JSON 的 `arguments.game` 中 `--width/--height`、`--quickPlay*`
/// 等参数通过 feature 规则（`has_custom_resolution`、`is_quick_play_singleplayer` 等）
/// 控制是否注入；`features` 传入这些 feature 的实际取值，缺失视为 `false`。
pub fn check_rules_with_features(
    rules: &Option<Vec<serde_json::Value>>,
    features: &[(&str, bool)],
) -> bool {
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

        if let Some(feats) = rule.get("features").and_then(|f| f.as_object()) {
            for (name, required_val) in feats {
                let required_val = required_val.as_bool().unwrap_or(false);
                let has = features
                    .iter()
                    .find(|(n, _)| *n == name.as_str())
                    .map(|(_, v)| *v)
                    .unwrap_or(false);
                if has != required_val {
                    is_right_rule = false;
                }
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
