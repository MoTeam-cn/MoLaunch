//! 客户端标识（ClientType / UA）工具
//!
//! 生成统一请求 User-Agent（`Molaunch/{主版本}.{clientType}`），
//! clientType 由平台编码与预发布后缀渠道码推导。

/// 预发布后缀 → 渠道码（clientType 个位）
///
/// 规则（语义化版本预发布标识）：
/// - 无后缀 → 0 正式版
/// - `-rc`   → 1 灰度版（Release Candidate，接近正式）
/// - `-beta` → 2 内测版
/// - `-alpha`/`-dev` → 3 开发版
/// - `-nightly` → 4 每日构建
/// - 未知后缀 → 3 开发版（防御性兜底）
fn channel_code(version: &str) -> u8 {
    let suffix = version.split('-').nth(1).unwrap_or("").to_lowercase();
    let suffix = suffix.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');
    if suffix.is_empty() {
        return 0;
    }
    if suffix.starts_with("rc") {
        return 1;
    }
    if suffix.starts_with("beta") {
        return 2;
    }
    if suffix.starts_with("alpha") || suffix.starts_with("dev") {
        return 3;
    }
    if suffix.starts_with("nightly") {
        return 4;
    }
    3
}

/// 平台/架构 → 平台码（clientType 十位）
///
/// 由编译时目标平台推导，覆盖 docs/client.md 的全部桌面 + 移动端编码。
/// 未知平台返回 0（理论上不会发生，防御性兜底）。
///
/// 用 `cfg!` 宏（编译期求值的 bool）而非 `#[cfg]` + return 链：
/// 后者在当前编译目标下会把结尾兜底分支判为"不可达"，前者始终可达。
fn platform_code() -> u8 {
    if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        1
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86") {
        2
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "aarch64") {
        3
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        4
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        5
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        6
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        7
    } else if cfg!(target_os = "android") {
        8
    } else if cfg!(target_os = "ios") {
        9
    } else {
        0
    }
}

/// 计算完整 clientType（两位数字：平台码 * 10 + 渠道码）
pub fn client_type(version: &str) -> u8 {
    platform_code() * 10 + channel_code(version)
}

/// 提取主版本号（去掉预发布后缀）
///
/// `0.1.0-beta.1` → `0.1.0`；`1.0.0-rc1` → `1.0.0`；`1.0.0` → `1.0.0`
fn main_version(version: &str) -> &str {
    version.split('-').next().unwrap_or(version)
}

/// 构建统一 User-Agent：`Molaunch/{主版本}.{clientType}`
///
/// 例：`Molaunch/1.0.0.10`（Windows x86_64 正式版）
pub fn user_agent() -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        "Molaunch/{}.{}",
        main_version(version),
        client_type(version)
    )
}

#[cfg(test)]
#[path = "client_type_tests.rs"]
mod tests;
