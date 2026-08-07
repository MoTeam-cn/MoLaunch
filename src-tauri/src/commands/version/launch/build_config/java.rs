//! 启动配置中的 Java 解析。

use crate::minecraft::version::setup::VersionSetup;

/// 解析版本独立设置与前端覆盖后的 Java 配置。
pub(super) fn resolve_java(
    setup: &VersionSetup,
    java_path: Option<String>,
) -> (Option<String>, Option<String>, u32, u32) {
    let resolved_java = java_path.or_else(|| {
        let mode = setup.java.java_mode.as_deref().unwrap_or("").trim();
        if mode.eq_ignore_ascii_case("custom") {
            setup.java.java_path.clone().filter(|s| !s.is_empty())
        } else {
            None
        }
    });

    (
        resolved_java,
        setup.java.java_mode.clone(),
        setup.java.java_version_min.unwrap_or(0),
        setup.java.java_version_max.unwrap_or(0),
    )
}
