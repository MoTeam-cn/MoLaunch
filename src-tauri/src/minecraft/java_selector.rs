//! Java 版本选择算法模块
//! 参考 PCL2 的 Java 版本选择逻辑

use super::java::JavaRuntime;

/// 根据 MC 版本获取所需的最低 Java 版本
///
/// # 规则（参考 PCL2）
/// | MC 版本      | 最低 Java | 推荐 Java |
/// |-------------|----------|----------|
/// | 26+ (新格式) | Java 21  | Java 21  |
/// | 1.20.5+     | Java 21  | Java 21  |
/// | 1.18-1.20.4 | Java 17  | Java 17  |
/// | 1.17        | Java 16  | Java 17  |
/// | 1.12-1.16   | Java 8   | Java 8   |
/// | 1.5以下      | Java 8   | Java 8   |
pub fn get_required_java_version(mc_version: &str) -> u32 {
    let (min, _max) = get_java_version_range(mc_version, None);
    min.unwrap_or(8)
}

/// 根据 MC 版本和加载器获取 Java 版本约束区间（MinVer/MaxVer 双向约束，参考 PCL2）
///
/// # 参数
/// - `mc_version`: MC 版本号（如 "1.20.1"、"26.2"）
/// - `loader`: 加载器类型（"forge"/"neoforge"/"fabric"/"quilt"/"optifine"/"liteloader"），None 表示原版
///
/// # 返回
/// `(min, max)`，None 表示该方向无约束。多条件叠加取最严格区间（收紧取交）
pub fn get_java_version_range(mc_version: &str, loader: Option<&str>) -> (Option<u32>, Option<u32>) {
    let parts: Vec<&str> = mc_version.split('.').collect();
    let major: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    // 原版规则：初始化为无约束，逐步收紧
    let mut min: Option<u32> = None;
    let mut max: Option<u32> = None;

    let tighten_min = |cur: &mut Option<u32>, new: u32| {
        *cur = Some(cur.map_or(new, |c| c.max(new)));
    };
    let tighten_max = |cur: &mut Option<u32>, new: u32| {
        *cur = Some(cur.map_or(new, |c| c.min(new)));
    };

    // 新版本格式 (26+) 需要 Java 21
    if major >= 26 {
        tighten_min(&mut min, 21);
    } else if major == 1 {
        // 1.20.5+ 需要 Java 21
        if minor == 20 && patch >= 5 {
            tighten_min(&mut min, 21);
        } else if minor > 20 {
            // 1.21+ 需要 Java 21
            tighten_min(&mut min, 21);
        } else if minor >= 18 {
            // 1.18-1.20.4 需要 Java 17
            tighten_min(&mut min, 17);
        } else if minor == 17 {
            // 1.17 需要 Java 16
            tighten_min(&mut min, 16);
        } else if minor >= 12 {
            // 1.12-1.16 需要 Java 8
            tighten_min(&mut min, 8);
        } else if minor <= 5 {
            // 1.5.2 及更早最高兼容 Java 8
            tighten_max(&mut max, 8);
        } else {
            // 其他版本默认 Java 8
            tighten_min(&mut min, 8);
        }
    } else {
        // 未知格式，默认 Java 8
        tighten_min(&mut min, 8);
    }

    // 加载器覆盖规则（参考 PCL2 ModLaunch.vb:1198-1280）
    if let Some(loader) = loader {
        let loader_lower = loader.to_lowercase();
        match loader_lower.as_str() {
            "forge" => {
                // Forge 1.6.1-1.7.2 必须 Java 7
                if major == 1 && minor >= 6 && minor <= 7 {
                    tighten_min(&mut min, 7);
                    tighten_max(&mut max, 7);
                }
                // Forge <=1.12 最高 Java 8
                if major == 1 && minor <= 12 {
                    tighten_max(&mut max, 8);
                }
                // Forge 1.13-1.15: Java 8~15
                if major == 1 && minor >= 13 && minor <= 15 {
                    tighten_min(&mut min, 8);
                    tighten_max(&mut max, 15);
                }
                // Forge 1.16.5+ 默认走原版规则（Java 17+），不额外约束
            }
            "neoforge" => {
                // NeoForge 1.20.1 及 1.20.2 的 20.2.62-beta 之前最高 Java 21（实际与原版一致，不额外约束）
            }
            "fabric" => {
                // Fabric 1.15-1.16 最低 Java 8
                if major == 1 && minor >= 15 && minor <= 16 {
                    tighten_min(&mut min, 8);
                }
                // Fabric 1.18+ 最低 Java 17（原版规则已覆盖）
            }
            "optifine" => {
                // OptiFine <1.7 最高 Java 8
                if major == 1 && minor < 7 {
                    tighten_max(&mut max, 8);
                }
                // OptiFine 1.8-1.11 必须恰好 Java 8
                if major == 1 && minor >= 8 && minor <= 11 {
                    tighten_min(&mut min, 8);
                    tighten_max(&mut max, 8);
                }
                // OptiFine 1.12 最高 Java 8
                if major == 1 && minor == 12 {
                    tighten_max(&mut max, 8);
                }
            }
            "liteloader" => {
                // LiteLoader 最高 Java 8
                tighten_max(&mut max, 8);
            }
            _ => {}
        }
    }

    (min, max)
}

/// 从版本 JSON 读取 Mojang 官方 Java 版本要求（覆盖规则表）
///
/// 返回 Some(major_version) 若 JSON 中有 javaVersion.majorVersion 字段且 >= 22
/// （PCL2 阈值，参考 ModLaunch.vb:1184-1196）
pub fn get_mojang_java_requirement(version_json: &serde_json::Value) -> Option<u32> {
    let major = version_json
        .get("javaVersion")?
        .get("majorVersion")?
        .as_u64()?;
    if major >= 22 {
        Some(major as u32)
    } else {
        None
    }
}

/// 获取推荐的 Java 版本
pub fn get_recommended_java_version(mc_version: &str) -> u32 {
    let required = get_required_java_version(mc_version);
    match required {
        16 => 17, // 1.17 推荐 Java 17
        other => other,
    }
}

/// 将 Java 版本需求区间 `(min, max)` 描述为人类可读的中文文案。
/// 用于 Java 兼容性校验失败的提示信息（统一文案，避免多处复制粘贴）。
pub fn describe_java_requirement(min: Option<u32>, max: Option<u32>) -> String {
    match (min, max) {
        (Some(mn), Some(mx)) if mn == mx => format!("需要 Java {}", mn),
        (Some(mn), Some(mx)) => format!("需要 Java {}~{}", mn, mx),
        (Some(mn), None) => format!("至少需要 Java {}", mn),
        (None, Some(mx)) => format!("最高兼容到 Java {}", mx),
        _ => String::new(),
    }
}

/// 检查指定 Java 是否兼容 MC 版本需求
///
/// # 参数
/// - `java_major_version`: Java 大版本号
/// - `mc_version`: MC 版本号
/// - `loader`: 加载器类型（可选）
///
/// # 返回
/// - `Ok(())`: 兼容
/// - `Err((current, min, max))`: 不兼容，返回当前版本和需求区间
pub fn check_java_compatible(
    java_major_version: u32,
    mc_version: &str,
    loader: Option<&str>,
) -> Result<(), (u32, Option<u32>, Option<u32>)> {
    let (min, max) = get_java_version_range(mc_version, loader);
    if let Some(min_req) = min {
        if java_major_version < min_req {
            return Err((java_major_version, min, max));
        }
    }
    if let Some(max_req) = max {
        if java_major_version > max_req {
            return Err((java_major_version, min, max));
        }
    }
    Ok(())
}

/// 从 Java 列表中选择最佳 Java
///
/// # 参数
/// - `mc_version`: Minecraft 版本号
/// - `java_list`: 已检测的 Java 列表
/// - `user_java_path`: 用户手动指定的 Java 路径（可选）
///
/// # 返回
/// 选中的 Java 可执行文件路径
pub fn select_best_java(
    mc_version: &str,
    java_list: &[JavaRuntime],
    user_java_path: Option<&str>,
) -> Option<String> {
    select_best_java_with_loader(mc_version, None, java_list, user_java_path)
}

/// 从 Java 列表中选择最佳 Java（支持加载器约束）
pub fn select_best_java_with_loader(
    mc_version: &str,
    loader: Option<&str>,
    java_list: &[JavaRuntime],
    user_java_path: Option<&str>,
) -> Option<String> {
    let (min_req, max_req) = get_java_version_range(mc_version, loader);

    // 1. 用户手动指定的 Java 优先（仅校验最低要求，不阻断，与 PCL2 一致：警告但允许强制使用）
    if let Some(user_path) = user_java_path {
        if !user_path.is_empty() {
            let user_java = java_list.iter().find(|j| {
                j.executable.eq_ignore_ascii_case(user_path)
                    || j.path_folder.eq_ignore_ascii_case(user_path)
            });

            if let Some(java) = user_java {
                if check_java_compatible(java.major_version, mc_version, loader).is_ok() {
                    crate::log_info!(
                        "[JavaSelector] Using user-specified Java: {} (requires {}-{})",
                        java.version,
                        min_req.unwrap_or(0),
                        max_req.map(|m| m.to_string()).unwrap_or("∞".to_string())
                    );
                    return Some(java.executable.clone());
                } else {
                    crate::log_warn!(
                        "[JavaSelector] User-specified Java {} incompatible (requires {}-{})",
                        java.major_version,
                        min_req.unwrap_or(0),
                        max_req.map(|m| m.to_string()).unwrap_or("∞".to_string())
                    );
                }
            } else {
                crate::log_warn!(
                    "[JavaSelector] User-specified Java not found in detected list: {}",
                    user_path
                );
            }
        }
    }

    // 2. 自动选择最佳 Java
    let recommended = get_recommended_java_version(mc_version);

    crate::log_info!(
        "[JavaSelector] MC {} requires Java {}-{} (recommended: {})",
        mc_version,
        min_req.unwrap_or(0),
        max_req.map(|m| m.to_string()).unwrap_or("∞".to_string()),
        recommended
    );

    // 筛选满足 MinVer/MaxVer 双向约束的 Java
    let mut candidates: Vec<&JavaRuntime> = java_list
        .iter()
        .filter(|j| {
            let mut ok = true;
            if let Some(min) = min_req {
                ok &= j.major_version >= min;
            }
            if let Some(max) = max_req {
                ok &= j.major_version <= max;
            }
            ok
        })
        .collect();

    if candidates.is_empty() {
        crate::log_error!(
            "[JavaSelector] No Java found meeting requirement (need {}-{})",
            min_req.unwrap_or(0),
            max_req.map(|m| m.to_string()).unwrap_or("∞".to_string())
        );
        return None;
    }

    // 按优先级排序（参考 PCL2 权重系统）
    candidates.sort_by(|a, b| {
        // 1. 推荐版本优先
        let a_is_recommended = a.major_version == recommended;
        let b_is_recommended = b.major_version == recommended;
        if a_is_recommended != b_is_recommended {
            return b_is_recommended.cmp(&a_is_recommended);
        }

        // 2. 64 位优先
        if a.is_64bit != b.is_64bit {
            return b.is_64bit.cmp(&a.is_64bit);
        }

        // 3. JRE 优先（运行游戏无需 JDK，体积更小；与 select_best_from_candidates 一致）
        if a.is_jre != b.is_jre {
            return b.is_jre.cmp(&a.is_jre);
        }

        // 4. 版本权重排序
        let a_weight = get_java_version_weight(a.major_version);
        let b_weight = get_java_version_weight(b.major_version);
        b_weight.cmp(&a_weight)
    });

    let best = candidates[0];
    crate::log_info!(
        "[JavaSelector] Selected Java: {} ({}) - {}bit",
        best.version,
        best.executable,
        if best.is_64bit { "64" } else { "32" }
    );

    Some(best.executable.clone())
}

/// 获取用于安装加载器的 Java 路径
///
/// 安装加载器（Forge/NeoForge）通常需要 Java 8+
/// 优先选择 Java 8，其次选择任何可用的 Java
/// 注意：安装器需要 java.exe 而不是 javaw.exe（需要控制台输出）
pub fn get_java_for_installer(java_list: &[JavaRuntime]) -> Option<String> {
    crate::log_info!("[JavaSelector] Finding Java for installer...");

    // 辅助函数：将 javaw.exe 转换为 java.exe
    let to_java_exe = |path: &str| -> String {
        if path.ends_with("javaw.exe") {
            path.replace("javaw.exe", "java.exe")
        } else if path.ends_with("javaw") {
            path.replace("javaw", "java")
        } else {
            path.to_string()
        }
    };

    // 优先使用 Java 8（Forge 安装器兼容性最好）
    let java8_candidates: Vec<&JavaRuntime> =
        java_list.iter().filter(|j| j.major_version == 8).collect();

    if let Some(best) = select_best_from_candidates(&java8_candidates) {
        let java_path = to_java_exe(&best.executable);
        crate::log_info!(
            "[JavaSelector] Using Java 8 for installer: {} ({})",
            best.version,
            java_path
        );
        return Some(java_path);
    }

    // 其次使用 Java 11-17（兼容性较好）
    let mid_candidates: Vec<&JavaRuntime> = java_list
        .iter()
        .filter(|j| j.major_version >= 11 && j.major_version <= 17)
        .collect();

    if let Some(best) = select_best_from_candidates(&mid_candidates) {
        let java_path = to_java_exe(&best.executable);
        crate::log_info!(
            "[JavaSelector] Using Java {} for installer: {} ({})",
            best.major_version,
            best.version,
            java_path
        );
        return Some(java_path);
    }

    // 最后使用任何可用的 Java 8+
    let any_candidates: Vec<&JavaRuntime> =
        java_list.iter().filter(|j| j.major_version >= 8).collect();

    if let Some(best) = select_best_from_candidates(&any_candidates) {
        let java_path = to_java_exe(&best.executable);
        crate::log_info!(
            "[JavaSelector] Using Java {} for installer: {} ({})",
            best.major_version,
            best.version,
            java_path
        );
        return Some(java_path);
    }

    crate::log_error!("[JavaSelector] No suitable Java found for installer");
    None
}

/// 从候选列表中选择最佳 Java（内部辅助函数）
fn select_best_from_candidates<'a>(candidates: &[&'a JavaRuntime]) -> Option<&'a JavaRuntime> {
    if candidates.is_empty() {
        return None;
    }

    let mut sorted = candidates.to_vec();
    sorted.sort_by(|a, b| {
        // 64 位优先
        if a.is_64bit != b.is_64bit {
            return b.is_64bit.cmp(&a.is_64bit);
        }
        // JRE 优先（安装器不需要 JDK）
        if a.is_jre != b.is_jre {
            return b.is_jre.cmp(&a.is_jre);
        }
        // 版本权重
        let a_weight = get_java_version_weight(a.major_version);
        let b_weight = get_java_version_weight(b.major_version);
        b_weight.cmp(&a_weight)
    });

    sorted.first().map(|&j| j)
}

/// Java 版本权重（参考 PCL2）
pub fn get_java_version_weight(major_version: u32) -> u32 {
    match major_version {
        7 => 0,
        8 => 30, // Java 8 权重最高（兼容性最好）
        9 => 4,
        10 => 5,
        11 => 14,
        12 => 6,
        13 => 7,
        14 => 8,
        15 => 9,
        16 => 12,
        17 => 31, // Java 17 权重最高（新版本推荐）
        18 => 13,
        19 => 10,
        20 => 11,
        21 => 29, // Java 21 权重高（最新 LTS）
        _ => major_version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_required_java_version() {
        // 1.20.5+ -> Java 21
        assert_eq!(get_required_java_version("1.20.5"), 21);
        assert_eq!(get_required_java_version("1.20.6"), 21);
        assert_eq!(get_required_java_version("1.21"), 21);
        assert_eq!(get_required_java_version("1.21.1"), 21);

        // 1.18-1.20.4 -> Java 17
        assert_eq!(get_required_java_version("1.18"), 17);
        assert_eq!(get_required_java_version("1.18.2"), 17);
        assert_eq!(get_required_java_version("1.19.4"), 17);
        assert_eq!(get_required_java_version("1.20.4"), 17);

        // 1.17 -> Java 16
        assert_eq!(get_required_java_version("1.17"), 16);
        assert_eq!(get_required_java_version("1.17.1"), 16);

        // 1.12-1.16 -> Java 8
        assert_eq!(get_required_java_version("1.12"), 8);
        assert_eq!(get_required_java_version("1.12.2"), 8);
        assert_eq!(get_required_java_version("1.16.5"), 8);

        // 1.5 以下 -> Java 8
        assert_eq!(get_required_java_version("1.5"), 8);
        assert_eq!(get_required_java_version("1.4.7"), 8);
    }

    #[test]
    fn test_get_recommended_java_version() {
        assert_eq!(get_recommended_java_version("1.20.5"), 21);
        assert_eq!(get_recommended_java_version("1.18.2"), 17);
        assert_eq!(get_recommended_java_version("1.17"), 17); // 推荐 17 而非 16
        assert_eq!(get_recommended_java_version("1.16.5"), 8);
    }

    #[test]
    fn test_select_best_java_with_user_path() {
        let java_list = vec![
            JavaRuntime {
                executable: "C:\\Java8\\java.exe".to_string(),
                path_folder: "C:\\Java8".to_string(),
                is_user_import: false,
                version: "1.8.0_321".to_string(),
                major_version: 8,
                is_jre: true,
                is_64bit: true,
            },
            JavaRuntime {
                executable: "C:\\Java17\\java.exe".to_string(),
                path_folder: "C:\\Java17".to_string(),
                is_user_import: false,
                version: "17.0.2".to_string(),
                major_version: 17,
                is_jre: true,
                is_64bit: true,
            },
        ];

        // 用户指定 Java 8
        let result = select_best_java("1.16.5", &java_list, Some("C:\\Java8\\java.exe"));
        assert_eq!(result, Some("C:\\Java8\\java.exe".to_string()));

        // 用户指定的 Java 不满足要求（MC 1.20.5 需要 Java 21，但列表中没有）
        let result = select_best_java("1.20.5", &java_list, Some("C:\\Java8\\java.exe"));
        assert_eq!(result, None); // 没有满足要求的 Java
    }

    #[test]
    fn test_select_best_java_auto() {
        let java_list = vec![
            JavaRuntime {
                executable: "C:\\Java8\\java.exe".to_string(),
                path_folder: "C:\\Java8".to_string(),
                is_user_import: false,
                version: "1.8.0_321".to_string(),
                major_version: 8,
                is_jre: true,
                is_64bit: true,
            },
            JavaRuntime {
                executable: "C:\\Java17\\java.exe".to_string(),
                path_folder: "C:\\Java17".to_string(),
                is_user_import: false,
                version: "17.0.2".to_string(),
                major_version: 17,
                is_jre: true,
                is_64bit: true,
            },
            JavaRuntime {
                executable: "C:\\Java21\\java.exe".to_string(),
                path_folder: "C:\\Java21".to_string(),
                is_user_import: false,
                version: "21.0.1".to_string(),
                major_version: 21,
                is_jre: true,
                is_64bit: true,
            },
        ];

        // MC 1.16.5 需要 Java 8
        let result = select_best_java("1.16.5", &java_list, None);
        assert_eq!(result, Some("C:\\Java8\\java.exe".to_string()));

        // MC 1.18.2 需要 Java 17
        let result = select_best_java("1.18.2", &java_list, None);
        assert_eq!(result, Some("C:\\Java17\\java.exe".to_string()));

        // MC 1.20.5 需要 Java 21
        let result = select_best_java("1.20.5", &java_list, None);
        assert_eq!(result, Some("C:\\Java21\\java.exe".to_string()));
    }
}
