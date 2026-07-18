//! 资源管理模块
//!
//! 参考 PCL2 的 Resources.resx + ExtractResources 机制：
//! 所有外部资源（文本模板、二进制 jar）在编译时通过 include_str!/include_bytes!
//! 嵌入二进制，运行时零文件 IO 读取；二进制资源释放时带 sha256 校验，
//! 只在目标文件不存在或 hash 不匹配时写入，避免每次启动重复写盘。
//!
//! 取代了原先基于 env!("CARGO_MANIFEST_DIR") 拼路径的实现——
//! 那种方式在打包到用户机器后路径不存在，属于发布版 bug。

use crate::{log_info, log_warn};
use sha2::{Digest, Sha256};
use std::path::Path;

/// 嵌入的文本资源内容
///
/// 编译时通过 include_str! 把 resources/ 下的文件内容直接打进二进制。
/// 新增文本资源时，在此 match 中追加一条分支即可。
fn embedded_text(path: &str) -> Option<&'static str> {
    match path {
        "defaults/config.ini" => Some(include_str!("../resources/defaults/config.ini")),
        "defaults/instance.ini" => Some(include_str!("../resources/defaults/instance.ini")),
        "defaults/setup.ini" => Some(include_str!("../resources/defaults/setup.ini")),
        "moddata.txt" => Some(include_str!("../resources/moddata.txt")),
        _ => None,
    }
}

/// 嵌入的二进制资源内容
///
/// 编译时通过 include_bytes! 把 resources/ 下的二进制文件直接打进二进制。
/// 新增二进制资源时，在此 match 中追加一条分支即可。
fn embedded_bytes(path: &str) -> Option<&'static [u8]> {
    match path {
        "forge-installer.jar" => Some(include_bytes!("../resources/forge-installer.jar")),
        "java-wrapper.jar" => Some(include_bytes!("../resources/java-wrapper.jar")),
        "lwjgl-unsafe-agent.jar" => Some(include_bytes!("../resources/lwjgl-unsafe-agent.jar")),
        _ => None,
    }
}

/// 读取文本资源内容
///
/// 资源在编译时已嵌入二进制，运行时直接返回，不做任何文件 IO。
pub fn read_resource(relative_path: &str) -> anyhow::Result<String> {
    match embedded_text(relative_path) {
        Some(content) => Ok(content.to_string()),
        None => Err(anyhow::anyhow!(
            "未注册的文本资源: {}（请在 resources.rs 的 embedded_text 中登记）",
            relative_path
        )),
    }
}

/// 读取二进制资源内容
///
/// 资源在编译时已嵌入二进制，运行时直接返回，不做任何文件 IO。
pub fn read_resource_bytes(relative_path: &str) -> anyhow::Result<Vec<u8>> {
    match embedded_bytes(relative_path) {
        Some(content) => Ok(content.to_vec()),
        None => Err(anyhow::anyhow!(
            "未注册的二进制资源: {}（请在 resources.rs 的 embedded_bytes 中登记）",
            relative_path
        )),
    }
}

/// 计算数据的 sha256 十六进制摘要
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// 释放二进制资源到目标路径（带 sha256 校验）
///
/// 参考 PCL2 的 ExtractResources：只在目标文件不存在或 sha256 不匹配时写入。
/// 这样每次启动不会因为重复写大文件而拖慢，也避免杀软误报频繁触发。
///
/// 释放成功后会在同目录写一个 `{name}.sha256` 校验文件，用于下次启动比对。
pub fn extract_resource(resource_path: &str, target_path: &Path) -> anyhow::Result<()> {
    let content = read_resource_bytes(resource_path)?;
    let expected_hash = sha256_hex(&content);

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 校验文件路径：与目标同目录，后缀 .sha256
    let hash_path = target_path.with_extension("sha256");

    // 命中缓存：文件存在 + 校验文件存在 + hash 一致 → 跳过写入
    // 注意：必须同时检查 target 和 hash 文件存在，否则会出现
    // "目标文件不存在但 hash 文件残留"时读到旧 hash 触发"不匹配"警告的情况
    let target_exists = target_path.exists();
    let hash_exists = hash_path.exists();
    if target_exists && hash_exists {
        let cached_hash = std::fs::read_to_string(&hash_path).unwrap_or_default();
        if cached_hash.trim() == expected_hash {
            return Ok(());
        }
        log_warn!(
            "资源 {} 的缓存文件 hash 不匹配（期望 {}，实际 {}），重新释放",
            resource_path,
            expected_hash,
            cached_hash.trim()
        );
    } else if target_exists != hash_exists {
        // 文件和 hash 文件只存在一个：状态不一致，需要重新释放
        log_warn!(
            "资源 {} 的缓存状态不一致（target={}, hash={}），重新释放",
            resource_path,
            target_exists,
            hash_exists
        );
    }
    // 两者都不存在：首次释放，不打印警告

    std::fs::write(target_path, &content)?;
    std::fs::write(&hash_path, &expected_hash)?;
    log_info!(
        "释放嵌入资源: {} -> {} (sha256={})",
        resource_path,
        target_path.display(),
        &expected_hash[..12]
    );
    Ok(())
}
