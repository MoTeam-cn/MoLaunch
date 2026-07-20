//! Java 选择算法模块

use crate::minecraft::java::JavaRuntime;

use super::compat::{check_java_compatible, filter_compatible_javas};
use super::rules::{get_java_version_range, get_recommended_java_version};
use super::weight::get_java_version_weight;

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
///
/// 选择流程：
/// 1. 若用户指定了 Java 路径，优先尝试使用（仅校验最低要求，不阻断）
/// 2. 否则自动筛选满足 MinVer/MaxVer 双向约束的候选，按推荐版本/64位/JRE/权重排序
pub fn select_best_java_with_loader(
    mc_version: &str,
    loader: Option<&str>,
    java_list: &[JavaRuntime],
    user_java_path: Option<&str>,
) -> Option<String> {
    let (min_req, max_req) = get_java_version_range(mc_version, loader);

    // 1. 用户手动指定的 Java 优先（仅校验最低要求，不阻断：警告但允许强制使用）
    if let Some(path) = try_user_specified_java(
        user_java_path,
        java_list,
        mc_version,
        loader,
        min_req,
        max_req,
    ) {
        return Some(path);
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

    let mut candidates = filter_compatible_javas(java_list, min_req, max_req);

    if candidates.is_empty() {
        crate::log_error!(
            "[JavaSelector] No Java found meeting requirement (need {}-{})",
            min_req.unwrap_or(0),
            max_req.map(|m| m.to_string()).unwrap_or("∞".to_string())
        );
        return None;
    }

    // 按优先级排序
    sort_candidates_by_priority(&mut candidates, recommended);

    let best = candidates[0];
    crate::log_info!(
        "[JavaSelector] Selected Java: {} ({}) - {}bit",
        best.version,
        best.executable,
        if best.is_64bit { "64" } else { "32" }
    );

    Some(best.executable.clone())
}

/// 尝试使用用户手动指定的 Java
///
/// 返回 `Some(path)` 表示命中且通过兼容性校验；`None` 表示未命中或未通过校验，
/// 调用方应继续走自动选择流程。
fn try_user_specified_java(
    user_java_path: Option<&str>,
    java_list: &[JavaRuntime],
    mc_version: &str,
    loader: Option<&str>,
    min_req: Option<u32>,
    max_req: Option<u32>,
) -> Option<String> {
    let user_path = user_java_path?;
    if user_path.is_empty() {
        return None;
    }

    let user_java = java_list.iter().find(|j| {
        j.executable.eq_ignore_ascii_case(user_path)
            || j.path_folder.eq_ignore_ascii_case(user_path)
    });

    let java = match user_java {
        Some(java) => java,
        None => {
            crate::log_warn!(
                "[JavaSelector] User-specified Java not found in detected list: {}",
                user_path
            );
            return None;
        }
    };

    if check_java_compatible(java.major_version, mc_version, loader).is_ok() {
        crate::log_info!(
            "[JavaSelector] Using user-specified Java: {} (requires {}-{})",
            java.version,
            min_req.unwrap_or(0),
            max_req.map(|m| m.to_string()).unwrap_or("∞".to_string())
        );
        Some(java.executable.clone())
    } else {
        crate::log_warn!(
            "[JavaSelector] User-specified Java {} incompatible (requires {}-{})",
            java.major_version,
            min_req.unwrap_or(0),
            max_req.map(|m| m.to_string()).unwrap_or("∞".to_string())
        );
        None
    }
}

/// 按优先级对候选 Java 排序
///
/// 排序优先级（高的排前）：
/// 1. 推荐版本优先
/// 2. 64 位优先
/// 3. JRE 优先（运行游戏无需 JDK，体积更小）
/// 4. 版本权重排序
fn sort_candidates_by_priority(candidates: &mut [&JavaRuntime], recommended: u32) {
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
}
