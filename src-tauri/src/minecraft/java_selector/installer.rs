//! 加载器安装器 Java 选择
//!
//! 安装器（注入器）本身对 Java 要求不高，但它在安装过程中会**调用 processors**
//! 去重写 jar（AutoRenamingTool 等），那些工具是按目标版本的 Java 编译的。
//! 用 Java 8 去跑新版 NeoForge 的 processor，只会得到 UnsupportedClassVersionError，
//! 安装产物（如 client-*-srg.jar）就不会生成——而安装器可能照样返回成功。
//!
//! 所以这里按目标 MC 版本决定**最低 Java 要求**，不再固定优先 Java 8。

use crate::minecraft::java::JavaRuntime;

use super::weight::get_java_version_weight;

/// 目标 MC 版本要求的最低 Java 主版本
///
/// - 新版本号体系（如 26.x）与 1.21+：Java 21
/// - 1.20.5 ~ 1.20.6：Java 21（mojmap 化之后要求）
/// - 1.18 ~ 1.20.4：Java 17
/// - 更早：Java 8
pub fn required_java_major(mc_version: &str) -> u32 {
    let parts: Vec<u32> = mc_version
        .split(['.', '-', '_'])
        .map_while(|part| part.parse::<u32>().ok())
        .collect();
    let major = parts.first().copied().unwrap_or(1);
    if major >= 20 {
        // 26.3 这类新版本号体系，早已是 Java 21 起步
        return 21;
    }
    let minor = parts.get(1).copied().unwrap_or(0);
    let patch = parts.get(2).copied().unwrap_or(0);
    if minor >= 21 {
        return 21;
    }
    if minor == 20 && patch >= 5 {
        return 21;
    }
    if minor >= 18 {
        return 17;
    }
    8
}

fn to_java_exe(path: &str) -> String {
    if path.ends_with("javaw.exe") {
        path.replace("javaw.exe", "java.exe")
    } else if path.ends_with("javaw") {
        path.replace("javaw", "java")
    } else {
        path.to_string()
    }
}

/// 取满足最低要求、且版本**最接近**要求的 Java
///
/// 不用最高的：够用就行。装 1.12.2 的 Forge 拿一个 Java 25 去跑，
/// 反而会踩到新旧模块系统的差异。
pub fn get_java_for_installer_with(java_list: &[JavaRuntime], min_major: u32) -> Option<String> {
    crate::log_info!(
        "[JavaSelector] Finding Java for installer (requires >= {})...",
        min_major
    );
    let mut candidates: Vec<&JavaRuntime> = java_list
        .iter()
        .filter(|java| java.major_version >= min_major)
        .collect();
    if candidates.is_empty() {
        crate::log_error!(
            "[JavaSelector] No Java >= {} found for installer",
            min_major
        );
        return None;
    }
    candidates.sort_by_key(|java| java.major_version);
    let lowest = candidates[0].major_version;
    let same_major: Vec<&JavaRuntime> = candidates
        .into_iter()
        .filter(|java| java.major_version == lowest)
        .collect();
    let best = select_best_from_candidates(&same_major)?;
    let java_path = to_java_exe(&best.executable);
    crate::log_info!(
        "[JavaSelector] Using Java {} for installer: {}",
        best.version,
        java_path
    );
    Some(java_path)
}

/// 兼容旧调用：只要 Java 8 及以上
pub fn get_java_for_installer(java_list: &[JavaRuntime]) -> Option<String> {
    get_java_for_installer_with(java_list, 8)
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
        // 优先 JDK：processor 里可能要用 javap 之类的工具
        if a.is_jre != b.is_jre {
            return a.is_jre.cmp(&b.is_jre);
        }
        // 版本权重
        let a_weight = get_java_version_weight(a.major_version);
        let b_weight = get_java_version_weight(b.major_version);
        b_weight.cmp(&a_weight)
    });

    sorted.first().copied()
}

#[cfg(test)]
#[path = "installer_test.rs"]
mod installer_test;
