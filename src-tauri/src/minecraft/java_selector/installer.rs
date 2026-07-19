//! 加载器安装器 Java 选择模块
//!
//! 安装加载器（Forge/NeoForge）通常需要 Java 8+
//! 注意：安装器需要 java.exe 而不是 javaw.exe（需要控制台输出）

use crate::minecraft::java::JavaRuntime;

use super::weight::get_java_version_weight;

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
pub(super) fn select_best_from_candidates<'a>(
    candidates: &[&'a JavaRuntime],
) -> Option<&'a JavaRuntime> {
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
