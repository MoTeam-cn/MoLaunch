//! Java 兼容性校验模块

use crate::minecraft::java::JavaRuntime;

use super::rules::get_java_version_range;

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

/// 筛选满足 MinVer/MaxVer 双向约束的 Java 列表
///
/// 返回候选 Java 引用列表（保留原始顺序）。
pub fn filter_compatible_javas(
    java_list: &[JavaRuntime],
    min_req: Option<u32>,
    max_req: Option<u32>,
) -> Vec<&JavaRuntime> {
    java_list
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
        .collect()
}
