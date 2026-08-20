//! frpc 二进制入口编排（ensure_frpc）：按厂商分发方式路由到 system_default / external

use super::super::provider::{
    get_frpc_path_for_provider, is_external_frpc_ready, read_provider_frpc_version,
    read_provider_manifest, write_provider_frpc_version, SYSTEM_DEFAULT_ID,
};
use super::{external, system_default};
use crate::log_info;
use crate::state::AppState;

/// 下载 frpc 二进制
///
/// `provider_id` 为 None 或 `system-default` 时走系统默认厂商下载逻辑
/// （GitHub API 取最新版本 + 镜像竞速下载）。
/// 外部厂商根据 manifest.binary.distribution 处理：
/// - bundled: 仅校验文件存在（厂商包自带 frpc）
/// - url: 从配置的 URL 下载（HTTPS + 域名白名单 + SHA256 校验）
///
/// frpc 更新判断：以 manifest.binary.frpc_version 为准（如 "0.51.3"）。
/// - 记录版本与 manifest 一致 → 视为就绪，不重复下载/替换
/// - 文件缺失或版本不一致 → 执行更新（url 重下 / bundled 提示重装）
///
/// `force=true` 时跳过就绪检查强制重新下载（系统默认厂商用于「有新版本」更新按钮）。
pub async fn ensure_frpc(
    state: &AppState,
    provider_id: Option<String>,
    force: bool,
) -> Result<String, String> {
    let pid = provider_id.unwrap_or_else(|| SYSTEM_DEFAULT_ID.to_string());
    if pid == SYSTEM_DEFAULT_ID {
        return system_default::ensure_system_default_frpc(state, force).await;
    }
    let manifest = read_provider_manifest(&pid)?;

    // frpc 版本比对：记录与 manifest 一致则直接就绪（文件在即可）
    let declared_version = manifest.binary.frpc_version.as_deref();
    let recorded_version = read_provider_frpc_version(&pid);
    let version_matches = match declared_version {
        Some(v) => recorded_version.as_deref() == Some(v),
        // manifest 未注明 frpc 版本：退化为旧行为（仅检查文件存在）
        None => true,
    };

    let ready = is_external_frpc_ready(&pid, &manifest);
    if ready && version_matches {
        let path = get_frpc_path_for_provider(&pid)?;
        return Ok(format!("frpc 已就绪: {}", path.display()));
    }
    if ready && !version_matches {
        log_info!(
            "[Frp] 厂商 {} frpc 版本变化 ({} -> {})，执行更新",
            pid,
            recorded_version.unwrap_or_else(|| "未知".to_string()),
            declared_version.unwrap_or("未知")
        );
    }
    // frpc 未就绪：bundled 无法补下，仅 url 可下载
    match manifest.binary.distribution.as_str() {
        "bundled" => {
            if ready && !version_matches {
                return Err(format!(
                    "厂商 {} 的 frpc 版本已变化（{}），请重新安装厂商包以更新 frpc",
                    pid,
                    declared_version.unwrap_or("未知")
                ));
            }
            Err(format!("厂商 {} 的 frpc 二进制缺失，请重新安装厂商包", pid))
        }
        "url" => {
            let path = external::ensure_external_frpc(&pid, &manifest).await?;
            // 下载成功后将 manifest 声明的 frpc 版本写入记录
            if let Some(v) = declared_version {
                write_provider_frpc_version(&pid, v);
            }
            Ok(path)
        }
        other => Err(format!("厂商 {} 使用不支持的分发方式: {}", pid, other)),
    }
}
