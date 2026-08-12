//! frpc 进程构造、环境隔离与启动。

use crate::commands::frp::provider;
use crate::commands::frp::{ensure_dir, frp_logs_dir, Tunnel};
use crate::log_debug;
use crate::log_info;

use super::prepare::prepare_config;

pub(super) struct SpawnedFrpc {
    pub(super) child: tokio::process::Child,
    pub(super) log_path: std::path::PathBuf,
    pub(super) tunnel: Tunnel,
}

pub(super) async fn spawn_frpc(
    state: &crate::state::AppState,
    tunnel: Tunnel,
) -> Result<SpawnedFrpc, String> {
    crate::commands::frp::binary::ensure_frpc(state, Some(tunnel.provider_id.clone())).await?;
    let frpc_path = provider::get_frpc_path_for_provider(&tunnel.provider_id)?;
    if !frpc_path.exists() {
        return Err(format!("frpc 二进制不存在: {}", frpc_path.display()));
    }

    let launch_mode = provider::read_provider_manifest(&tunnel.provider_id)
        .ok()
        .and_then(|m| m.binary.launch)
        .filter(|l| l.mode.eq_ignore_ascii_case("command"));
    let config_path = if launch_mode.is_none() {
        Some(prepare_config(state, &tunnel).await?)
    } else {
        None
    };

    let logs_dir = frp_logs_dir();
    ensure_dir(&logs_dir)?;
    let log_path = logs_dir.join(format!("{}.log", tunnel.id));
    std::fs::write(&log_path, "").ok();
    log_info!(
        "[Frp] 启动隧道: {} ({}), frpc={}, config={}",
        tunnel.name,
        tunnel.id,
        frpc_path.display(),
        config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "命令直连模式".to_string())
    );

    let mut cmd = tokio::process::Command::new(&frpc_path);
    if let Some(launch) = launch_mode {
        let remote_id = tunnel.remote_tunnel_id.as_deref().unwrap_or(&tunnel.id);
        let token = tunnel.token.as_deref().unwrap_or("");
        if !token.is_empty()
            && (token.starts_with('-')
                || token.chars().any(|c| c.is_whitespace())
                || token.contains(',')
                || token.contains('='))
        {
            return Err("Token 非法，已拒绝启动隧道".to_string());
        }
        let template = launch
            .command
            .as_deref()
            .unwrap_or("")
            .to_string()
            .replace("{frpc}", &frpc_path.to_string_lossy())
            .replace("{tunnelId}", remote_id);
        log_info!("[Frp] 使用厂商命令模式启动: {}", template);
        for arg in build_command_args(&template, token) {
            cmd.arg(arg);
        }
    } else {
        cmd.arg("-c").arg(config_path.as_ref().unwrap());
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.stdin(std::process::Stdio::null());
    configure_environment(&mut cmd);

    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let child = cmd.spawn().map_err(|e| format!("启动 frpc 失败: {}", e))?;
    Ok(SpawnedFrpc {
        child,
        log_path,
        tunnel,
    })
}

fn configure_environment(cmd: &mut tokio::process::Command) {
    let keep_keys = [
        "PATH",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "no_proxy",
        "SystemRoot",
        "SystemDrive",
        "SYSTEMROOT",
        "WINDIR",
        "TEMP",
        "TMP",
        "ComSpec",
        "USERPROFILE",
    ];
    let mut kept_envs = Vec::new();
    for key in keep_keys {
        if let Ok(val) = std::env::var(key) {
            kept_envs.push((key.to_string(), val));
        }
    }
    cmd.env_clear();
    for (key, value) in &kept_envs {
        cmd.env(key, value);
    }
    let env_desc: Vec<String> = kept_envs
        .iter()
        .map(|(key, value)| {
            let is_proxy = key.contains("PROXY") || key.contains("proxy");
            let shown = if is_proxy && !value.is_empty() {
                "<已设置，值已脱敏>".to_string()
            } else if is_proxy {
                "<未设置>".to_string()
            } else {
                value.clone()
            };
            format!("{}={}", key, shown)
        })
        .collect();
    log_debug!("[Frp] 传给 frpc 的环境变量: {}", env_desc.join("; "));
}

/// 构造命令模式启动参数（跳过首个二进制路径词，逐词内联替换 `{token}`）
///
/// `{token}` 是参数内联占位符（如 Lolia `-t 17062:{token}`：token 必须与
/// 端口拼接为同一参数，不能拆成独立参数），token 为空时整词替换为空则跳过。
fn build_command_args(template: &str, token: &str) -> Vec<String> {
    let mut words = template.split_whitespace();
    let _ = words.next();
    words
        .filter_map(|word| {
            if word.contains("{token}") {
                let substituted = word.replace("{token}", token);
                if substituted.is_empty() {
                    None
                } else {
                    Some(substituted)
                }
            } else {
                Some(word.to_string())
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "spawn_test.rs"]
mod tests;
