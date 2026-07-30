//! Java 选择模块（基于版本区间的简单选择）
//!
//! 复杂的 MC 版本感知选择逻辑在 `minecraft::java_selector`，此处仅做 MinVer/MaxVer 筛选并复用其权重系统

use super::JavaRuntime;

/// 从 Java 列表中选择最佳 Java（基于版本区间）
///
/// - `min_version`: 最低 Java 大版本（可选）
/// - `max_version`: 最高 Java 大版本（可选）
///
/// 排序优先级：64位优先 → JRE优先 → 版本权重（复用 java_selector 权重系统）
pub fn select_best_java(
    java_list: &[JavaRuntime],
    min_version: Option<u32>,
    max_version: Option<u32>,
) -> Option<&JavaRuntime> {
    let mut candidates: Vec<&JavaRuntime> = java_list.iter().collect();

    if let Some(min) = min_version {
        candidates.retain(|java| java.major_version >= min);
    }
    if let Some(max) = max_version {
        candidates.retain(|java| java.major_version <= max);
    }

    // 排序：64位优先，JRE优先，版本权重
    candidates.sort_by(|a, b| {
        if a.is_64bit != b.is_64bit {
            return b.is_64bit.cmp(&a.is_64bit);
        }
        if a.is_jre != b.is_jre {
            return b.is_jre.cmp(&a.is_jre);
        }
        let a_weight = crate::minecraft::java_selector::get_java_version_weight(a.major_version);
        let b_weight = crate::minecraft::java_selector::get_java_version_weight(b.major_version);
        b_weight.cmp(&a_weight)
    });

    candidates.first().map(|&java| java)
}
