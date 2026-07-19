//! 错误处理工具
//!
//! 提供 `log_err` 辅助函数，统一 `.map_err(|e| { log_error!(...); e.to_string() })` 样板。
//!
//! 用法：
//! ```rust,ignore
//! use crate::error_util::log_err;
//!
//! let versions = list_versions(&mc_version)
//!     .map_err(log_err("Failed to list versions"))?;
//! ```

/// 创建一个闭包，用于 `.map_err()` 中记录错误日志并转换为 `String`。
///
/// 消除项目中 40+ 处 `.map_err(|e| { log_error!("xxx: {}", e); e.to_string() })` 样板。
///
/// # 示例
///
/// ```rust,ignore
/// let data = fetch_url(url)
///     .map_err(log_err("Failed to fetch URL"))?;
/// ```
///
/// 等价于：
///
/// ```rust,ignore
/// let data = fetch_url(url).map_err(|e| {
///     log_error!("Failed to fetch URL: {}", e);
///     e.to_string()
/// })?;
/// ```
pub fn log_err<E: std::fmt::Display>(label: &str) -> impl FnOnce(E) -> String {
    let label = label.to_string();
    move |e: E| {
        crate::log_error!("{}: {}", label, e);
        e.to_string()
    }
}

/// 创建一个闭包，用于 `.map_err()` 中记录错误日志（带上下文）并转换为 `String`。
///
/// 与 `log_err` 的区别：额外接收一个 context 字符串，用于附加版本号、路径等信息。
///
/// # 示例
///
/// ```rust,ignore
/// let result = install_loader(version_id)
///     .map_err(log_err_with("Failed to install loader", &version_id))?;
/// ```
pub fn log_err_with<E: std::fmt::Display>(label: &str, context: &str) -> impl FnOnce(E) -> String {
    let label = label.to_string();
    let context = context.to_string();
    move |e: E| {
        crate::log_error!("{} ({}): {}", label, context, e);
        e.to_string()
    }
}
