//! 红石内核获取：运行时下载 + 本地缓存 + 编译期兜底。
//!
//! 服务端对内核做版本校验（旧内核会被直接拒绝），所以内核不能只靠编译期嵌入——
//! 那样对方每发一版 MoLaunch 都得跟着发版。获取顺序：
//! 1. AppData 缓存已有内核 → 直接用，不联网
//! 2. 否则从官方 /api/download/client 下载，校验后原子写入缓存
//! 3. 下载失败（服务不可用 / 该平台尚未发布）→ 回退编译期嵌入的兜底内核

use std::path::PathBuf;

/// 官方内核下载端点（kind=client 即启动器要的 hongshic；server 是中继 hongshis，webui 是图形壳）
const KERNEL_DOWNLOAD_BASE: &str = "https://hongshi.site/api/download/client";

/// 下载超时（内核不到 1MB，60s 足够）
const DOWNLOAD_TIMEOUT_SECS: u64 = 60;

/// 内核来源（用于日志与错误提示）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSource {
    /// 命中本地缓存
    Cache,
    /// 本次从官网下载
    Downloaded,
    /// 回退编译期嵌入的兜底内核
    Embedded,
}

impl KernelSource {
    /// 供日志与前端展示的来源标识
    pub fn as_str(self) -> &'static str {
        match self {
            KernelSource::Cache => "cache",
            KernelSource::Downloaded => "downloaded",
            KernelSource::Embedded => "embedded",
        }
    }
}

/// 官方是否仍为当前平台提供内核
///
/// 官方已停止提供 macOS 构建（下载页仍挂链接但实际 404），该平台判为不支持。
pub fn is_platform_supported() -> bool {
    !cfg!(target_os = "macos")
}

/// 当前平台对应的官方命名（platform, arch）
///
/// 官方取值：platform ∈ windows|linux|macos，arch ∈ amd64|arm64。
/// 未覆盖的组合返回 None（如 macOS 官方已停供、Windows arm64 未发布）。
pub fn platform_arch() -> Option<(&'static str, &'static str)> {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return Some(("windows", "amd64"));
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return Some(("linux", "amd64"));
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        return Some(("linux", "arm64"));
    }
    #[allow(unreachable_code)]
    None
}

/// 官方文件名：`{binary}-{platform}-{arch}`，仅 windows 追加 .exe
pub fn kernel_file_name() -> Option<String> {
    let (platform, arch) = platform_arch()?;
    let suffix = if platform == "windows" { ".exe" } else { "" };
    Some(format!("hongshic-{platform}-{arch}{suffix}"))
}

/// 内核缓存目录（AppData 下持久保存，避免每次启动重复下载）
pub fn kernel_cache_dir() -> Result<PathBuf, String> {
    crate::storage::appdata::ensure_appdata_subdir("hongshi")
}

/// 缓存中的内核路径
pub fn cached_kernel_path() -> Result<PathBuf, String> {
    let name = kernel_file_name().ok_or_else(|| "当前平台无官方红石内核".to_string())?;
    Ok(kernel_cache_dir()?.join(name))
}

/// 解析可用内核：缓存 → 下载 → 编译期兜底
///
/// 返回（内核路径，来源）。三条路径都不可用时返回带原因的错误。
pub async fn ensure_kernel() -> Result<(PathBuf, KernelSource), String> {
    // 0. 平台闸门：官方已停供的平台直接给出明确原因，不再走下载与兜底
    if !is_platform_supported() {
        return Err("红石联机已不再支持 macOS：官方已停止提供 macOS 内核".to_string());
    }

    // 1. 本地缓存
    if let Ok(path) = cached_kernel_path() {
        if path.is_file() {
            return Ok((path, KernelSource::Cache));
        }
    }

    // 2. 运行时下载
    match download_and_cache().await {
        Ok(path) => Ok((path, KernelSource::Downloaded)),
        Err(download_err) => {
            // 3. 回退编译期兜底（下载服务不可用或该平台未发布时）
            match crate::resources::extract_hongshi_core() {
                Ok(path) => {
                    crate::log_warn!(
                        "[Redstone] 内核下载失败，已回退内置兜底内核: {}",
                        download_err
                    );
                    Ok((path, KernelSource::Embedded))
                }
                Err(embed_err) => Err(format!(
                    "获取红石内核失败：下载失败（{}），内置兜底也不可用（{}）",
                    download_err, embed_err
                )),
            }
        }
    }
}

/// 下载内核并原子写入缓存
async fn download_and_cache() -> Result<PathBuf, String> {
    let (platform, arch) = platform_arch().ok_or_else(|| "当前平台无官方红石内核".to_string())?;
    let url = format!("{KERNEL_DOWNLOAD_BASE}?platform={platform}&arch={arch}");
    let bytes = download_kernel(&url, platform).await?;

    let path = cached_kernel_path()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("创建内核缓存目录失败: {e}"))?;
    }

    // 先写临时文件再改名：避免下载中断留下半个可执行文件被当成有效缓存
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &bytes).map_err(|e| format!("写入内核缓存失败: {e}"))?;
    set_executable(&tmp)?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("落盘内核缓存失败: {e}"))?;

    crate::log_info!(
        "[Redstone] 内核已下载并缓存: {} ({} 字节)",
        path.display(),
        bytes.len()
    );
    Ok(path)
}

/// 下载内核字节并校验确为目标平台可执行文件
async fn download_kernel(url: &str, platform: &str) -> Result<Vec<u8>, String> {
    let resp = crate::http::get_client()
        .get(url)
        .timeout(std::time::Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .send()
        .await
        .map_err(|e| format!("请求内核下载失败: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("内核下载返回 HTTP {}", status.as_u16()));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("读取内核数据失败: {e}"))?;
    if bytes.is_empty() {
        return Err("内核下载内容为空".to_string());
    }
    if !is_expected_executable(&bytes, platform) {
        // 官方在参数非法/构建未发布时会返回 JSON 或 HTML 错误体，防止把它写进缓存
        return Err("内核下载内容不是可执行文件（可能该平台尚未发布）".to_string());
    }
    Ok(bytes.to_vec())
}

/// 按平台校验可执行文件魔数
fn is_expected_executable(bytes: &[u8], platform: &str) -> bool {
    match platform {
        // PE：MZ
        "windows" => bytes.starts_with(b"MZ"),
        // ELF：0x7F 'E' 'L' 'F'
        "linux" => bytes.starts_with(b"\x7fELF"),
        // Mach-O：32/64 位、大端/小端四种头
        "macos" => {
            matches!(
                bytes.get(..4),
                Some([0xfe, 0xed, 0xfa, 0xce])
                    | Some([0xce, 0xfa, 0xed, 0xfe])
                    | Some([0xfe, 0xed, 0xfa, 0xcf])
                    | Some([0xcf, 0xfa, 0xed, 0xfe])
            )
        }
        _ => false,
    }
}

/// 非 Windows 平台补可执行位
fn set_executable(path: &std::path::Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("设置内核可执行权限失败: {e}"))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
#[path = "kernel_test.rs"]
mod kernel_test;
