//! Java 版本规则模块

/// 根据 MC 版本获取所需的最低 Java 版本
///
/// # 规则
/// | MC 版本 | 最低 / 推荐 Java |
/// |--------|----------------|
/// | 26+ 或 1.20.5+ | Java 21 |
/// | 1.18-1.20.4 | Java 17 |
/// | 1.17 | Java 16 / 推荐 17 |
/// | 1.12-1.16 及以下 | Java 8 |
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
pub fn get_java_version_range(
    mc_version: &str,
    loader: Option<&str>,
) -> (Option<u32>, Option<u32>) {
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
                if major == 1 && (6..=7).contains(&minor) {
                    tighten_min(&mut min, 7);
                    tighten_max(&mut max, 7);
                }
                // Forge <=1.12 最高 Java 8
                if major == 1 && minor <= 12 {
                    tighten_max(&mut max, 8);
                }
                // Forge 1.13-1.15: Java 8~15
                if major == 1 && (13..=15).contains(&minor) {
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
                if major == 1 && (15..=16).contains(&minor) {
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
                if major == 1 && (8..=11).contains(&minor) {
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
/// 加载器 × Java 的硬约束：返回 Some(原因) 表示「一定会崩，应阻断启动」
///
/// 依据：旧版 Fabric / Quilt 加载器（0.15 之前）在 Java 25 上会在引导阶段直接崩溃
/// （加载器自身的 ASM 版本读不懂 Java 25 的类文件）。新版加载器已适配，不拦。
/// 用户可在版本设置里开启「忽略 Java 兼容性警告」绕过本约束。
pub fn check_loader_java_compat(
    loader: &str,
    loader_version: Option<&str>,
    java_major: u32,
) -> Option<String> {
    let loader_lower = loader.to_lowercase();
    if loader_lower != "fabric" && loader_lower != "quilt" {
        return None;
    }
    if java_major < 25 {
        return None;
    }
    let version = loader_version?;
    let parsed = super::loader_version::parse_semver3(version)?;
    if parsed >= (0, 15, 0) {
        return None;
    }
    Some(format!(
        "{} {} 不支持 Java {}：Java 25 起收紧了原生访问限制，旧版加载器会在启动阶段直接崩溃。\n请在 版本设置 → 游戏 Java 改为 Java 21 或更低，或把加载器升级到 0.15 及以上。",
        loader_lower, version, java_major
    ))
}

/// Java 过新提示：返回 Some(文案) 表示「很可能不兼容，建议换版本」
///
/// Java 25 起收紧了原生访问限制（JEP 472）并移除了一批旧 API，Minecraft 1.x 系列
/// 使用的旧版 LWJGL 与加载器 ASM 未必适配。只提示不阻断——用户可能确实需要新 Java。
pub fn java_too_new_warning(mc_version: &str, java_major: u32) -> Option<String> {
    if java_major < 25 {
        return None;
    }
    // 仅对 1.x 系列提示；26.x 起（年.发布号）已适配新 JDK
    if !mc_version.starts_with("1.") {
        return None;
    }
    Some(format!(
        "当前 Java {} 对 Minecraft {} 可能过新（Java 25 起收紧原生访问限制），若启动失败请改用 Java 21 或更低版本",
        java_major, mc_version
    ))
}

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
