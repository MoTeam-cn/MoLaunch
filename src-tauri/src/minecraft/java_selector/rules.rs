//! Java 版本规则模块

/// 根据 MC 版本获取所需的最低 Java 版本
///
/// # 规则
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

/// 根据 MC 版本和加载器获取 Java 版本约束区间（MinVer/MaxVer 双向约束）
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

    // 加载器覆盖规则
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
/// （阈值参考：低于 22 视为旧版本号，不覆盖规则表）
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
