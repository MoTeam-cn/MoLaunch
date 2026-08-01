//! 安全沙箱：隧道配置校验 + 认证适配器脚本沙箱
//!
//! - `validate_tunnel`：校验用户输入的隧道参数，防止注入和非法值（§7.2）
//! - `run_auth_adapter`：沙箱化执行厂商认证适配器脚本（§7.5）

use super::provider::{read_provider_manifest, SYSTEM_DEFAULT_ID};
use super::tunnel::{CreateTunnelParams, UpdateTunnelParams};
use super::{validate_provider_id, ProcessPermissions, TunnelType};
use crate::commands::frp::providers_root;
use crate::commands::plugins::spawn::{
    is_command_allowed, truncate_output, ProcessResult, MAX_TIMEOUT_MS,
};
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// 校验创建隧道参数
pub fn validate_tunnel(params: &CreateTunnelParams) -> Result<(), String> {
    // 厂商 ID 校验（非空 + kebab-case 格式）
    let provider_id = params.provider_id.trim();
    if provider_id.is_empty() {
        return Err("厂商 ID 不能为空".to_string());
    }
    validate_provider_id(provider_id)?;

    // 名称校验
    let name = params.name.trim();
    if name.is_empty() {
        return Err("隧道名称不能为空".to_string());
    }
    if name.len() > 64 {
        return Err("隧道名称不能超过 64 字符".to_string());
    }
    // 禁止换行和引号（防止 TOML 注入）
    if name.contains('\n') || name.contains('\r') || name.contains('"') || name.contains('\\') {
        return Err("隧道名称包含非法字符".to_string());
    }

    // 服务端地址校验
    let server_addr = params.server_addr.trim();
    if server_addr.is_empty() {
        return Err("服务器地址不能为空".to_string());
    }
    if server_addr.len() > 255 {
        return Err("服务器地址过长".to_string());
    }
    // 禁止换行、引号、反斜杠（防止 TOML 注入和路径遍历）
    if server_addr.contains('\n')
        || server_addr.contains('\r')
        || server_addr.contains('"')
        || server_addr.contains('\\')
    {
        return Err("服务器地址包含非法字符".to_string());
    }
    // 禁止 file:// 等协议前缀
    if server_addr.contains("://") {
        return Err("服务器地址不能包含协议前缀".to_string());
    }

    // 本地 IP 校验（如有）
    if let Some(ref local_ip) = params.local_ip {
        let local_ip = local_ip.trim();
        if !local_ip.is_empty()
            && (local_ip.contains('\n') || local_ip.contains('\r') || local_ip.contains('"'))
        {
            return Err("本地 IP 包含非法字符".to_string());
        }
    }

    // 端口校验
    if params.server_port == 0 {
        return Err("服务器端口不能为 0".to_string());
    }
    if params.local_port == 0 {
        return Err("本地端口不能为 0".to_string());
    }
    // 禁止绑定特权端口（< 1024），防止 frpc 获取不必要的系统权限
    if params.local_port < 1024 {
        return Err("本地端口不能小于 1024".to_string());
    }
    if params.remote_port == 0 {
        return Err("远程端口不能为 0".to_string());
    }

    // Token 校验（如有）
    if let Some(ref token) = params.token {
        if !token.is_empty() {
            if token.contains('\n')
                || token.contains('\r')
                || token.contains('"')
                || token.contains('\\')
            {
                return Err("Token 包含非法字符".to_string());
            }
            if token.len() > 512 {
                return Err("Token 过长".to_string());
            }
        }
    }

    // 隧道类型校验
    match params.tunnel_type {
        TunnelType::Tcp | TunnelType::Udp => {}
    }

    // 网络白名单强制校验（厂商 manifest 的 networkPermissions + 内网地址检查）
    validate_network_permissions(params)?;

    Ok(())
}

/// 网络白名单强制校验
///
/// 对应设计文档 §7.2 配置校验。两项检查：
/// 1. 若厂商 `network_permissions.allow_custom_server=false`，`server_addr` 必须在
///    `allowed_servers` 白名单内（系统默认厂商无 manifest，允许自定义服务器）。
///    白名单项支持完整 `host:port` 匹配、host 匹配、以及 `*.example.com` 通配符
///    （复用 `binary::host_matches`，供平台动态节点厂商如 LoliaFrp 使用）。
/// 2. 非系统默认厂商禁止连接内网地址（10.0.0.0/8、172.16.0.0/12、192.168.0.0/16、
///    127.0.0.0/8），防止 SSRF。系统默认厂商豁免（用户自建 frps 可能位于内网）。
fn validate_network_permissions(params: &CreateTunnelParams) -> Result<(), String> {
    use super::binary::host_matches;
    use super::provider::{read_provider_manifest, SYSTEM_DEFAULT_ID};

    let is_system_default = params.provider_id == SYSTEM_DEFAULT_ID;

    if !is_system_default {
        // 读取厂商 manifest 的 networkPermissions
        let manifest = read_provider_manifest(&params.provider_id)
            .map_err(|e| format!("读取厂商清单失败: {}", e))?;

        if let Some(ref net_perm) = manifest.network_permissions {
            if !net_perm.allow_custom_server {
                let server_addr = params.server_addr.trim();
                let allowed = &net_perm.allowed_servers;
                // 白名单匹配：完整匹配、host 匹配、或 `*.domain` 通配符匹配
                let addr_host = server_addr.split(':').next().unwrap_or(server_addr);
                let matched = allowed.iter().any(|s| {
                    let s = s.trim();
                    if s == server_addr {
                        return true;
                    }
                    let s_host = s.split(':').next().unwrap_or(s);
                    s_host == addr_host || host_matches(addr_host, s_host)
                });
                if !matched {
                    return Err(format!(
                        "服务器地址 {} 不在厂商 {} 的允许列表内",
                        server_addr, params.provider_id
                    ));
                }
            }
        }
    }

    // 内网地址检查：非系统默认厂商禁止连接内网地址（防止 SSRF）
    if !is_system_default {
        let server_addr = params.server_addr.trim();
        if is_private_address(server_addr) {
            return Err("非系统默认厂商禁止连接内网地址".to_string());
        }
    }

    Ok(())
}

/// 判断地址是否为内网/回环地址
///
/// 支持 `host` 和 `host:port` 两种形式。非字面量 IP（域名）仅检查 `localhost`。
/// 覆盖范围：10.0.0.0/8、172.16.0.0/12、192.168.0.0/16（`Ipv4Addr::is_private`）、
/// 127.0.0.0/8（`Ipv4Addr::is_loopback`）。
fn is_private_address(addr: &str) -> bool {
    use std::net::{IpAddr, SocketAddr};
    // 优先按 SocketAddr 解析（处理 host:port），再按裸 IP 解析
    let ip = if let Ok(s) = addr.parse::<SocketAddr>() {
        Some(s.ip())
    } else {
        addr.parse::<IpAddr>().ok()
    };
    if let Some(ip) = ip {
        return match ip {
            IpAddr::V4(v4) => v4.is_private() || v4.is_loopback(),
            IpAddr::V6(v6) => v6.is_loopback(),
        };
    }
    // 非字面量 IP（域名）：仅检查 localhost
    addr.eq_ignore_ascii_case("localhost")
}

/// 校验更新隧道参数
///
/// `UpdateTunnelParams` 与 `CreateTunnelParams` 字段一致（仅多一个 `id`），
/// 转换为 `CreateTunnelParams` 后复用 `validate_tunnel` 的校验逻辑，避免重复实现规则。
pub fn validate_tunnel_update(p: &UpdateTunnelParams) -> Result<(), String> {
    let create = CreateTunnelParams {
        name: p.name.clone(),
        provider_id: p.provider_id.clone(),
        tunnel_type: p.tunnel_type.clone(),
        local_ip: p.local_ip.clone(),
        local_port: p.local_port,
        server_addr: p.server_addr.clone(),
        server_port: p.server_port,
        remote_port: p.remote_port,
        token: p.token.clone(),
        use_tls: p.use_tls,
    };
    validate_tunnel(&create)
}

// 认证适配器脚本沙箱（§7.5）
/// 沙箱化执行厂商认证适配器脚本（§7.5）
///
/// 流程：读取厂商 `process_permissions` → 校验命令在白名单（`spawn::is_command_allowed`）
/// → 工作目录锁定厂商目录 → 超时控制（默认 30s，最大 5min）→ 非 shell 执行防注入
/// → `env_clear()` 仅保留 PATH → stdout/stderr 各截断到 1MB。
/// 系统默认厂商不提供自定义脚本，直接拒绝。
pub async fn run_auth_adapter(
    provider_id: &str,
    command: String,
    args: Vec<String>,
) -> Result<ProcessResult, String> {
    use crate::log_debug;

    // 1. 系统默认厂商不提供自定义脚本
    if provider_id == SYSTEM_DEFAULT_ID {
        return Err("系统默认厂商不支持自定义认证适配器脚本".to_string());
    }

    // 2. 校验厂商 ID 合法性
    validate_provider_id(provider_id)?;

    // 3. 读取厂商 manifest 的 process_permissions
    let manifest = read_provider_manifest(provider_id)?;
    let proc_perms: &ProcessPermissions =
        manifest.process_permissions.as_ref().ok_or_else(|| {
            format!(
                "厂商 {} 未配置 processPermissions，禁止执行认证适配器脚本",
                provider_id
            )
        })?;

    if proc_perms.allowed_commands.is_empty() {
        return Err(format!(
            "厂商 {} 的 allowedCommands 白名单为空",
            provider_id
        ));
    }

    // 4. 校验命令在白名单内（复用 spawn::is_command_allowed）
    if !is_command_allowed(&command, &proc_perms.allowed_commands)? {
        return Err(format!(
            "命令不在厂商 {} 的白名单内: {}",
            provider_id, command
        ));
    }

    // 5. 校验超时上限（默认 30s，最大 5min）
    let timeout_ms = proc_perms.timeout_ms.min(MAX_TIMEOUT_MS);
    let timeout = Duration::from_millis(timeout_ms);

    // 6. 工作目录强制设为厂商目录
    let provider_dir = providers_root().join(provider_id);
    if !provider_dir.exists() {
        return Err(format!("厂商目录不存在: {}", provider_dir.display()));
    }

    log_debug!(
        "[Frp Sandbox] 执行认证适配器: provider={}, command={}, args={:?}, timeout={}ms",
        provider_id,
        command,
        args,
        timeout_ms
    );

    // 7. 构建 Command（非 shell 执行，防注入）
    let mut cmd = Command::new(&command);
    cmd.args(&args);
    cmd.current_dir(&provider_dir);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.stdin(std::process::Stdio::null());

    // 8. 清空环境变量，仅保留 PATH（防止敏感环境变量泄露）
    cmd.env_clear();
    if let Ok(path) = std::env::var("PATH") {
        cmd.env("PATH", path);
    }

    let start = Instant::now();

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动认证适配器失败 '{}': {}", command, e))?;

    let stdout_handle = child.stdout.take();
    let stderr_handle = child.stderr.take();

    // 9. 并行读取 stdout/stderr
    let stdout_task = tokio::spawn(async move {
        if let Some(mut stdout) = stdout_handle {
            let mut buf = Vec::new();
            stdout.read_to_end(&mut buf).await.ok();
            buf
        } else {
            Vec::new()
        }
    });

    let stderr_task = tokio::spawn(async move {
        if let Some(mut stderr) = stderr_handle {
            let mut buf = Vec::new();
            stderr.read_to_end(&mut buf).await.ok();
            buf
        } else {
            Vec::new()
        }
    });

    // 10. 超时包裹 child.wait()
    let (exit_code, timed_out) = match tokio::time::timeout(timeout, child.wait()).await {
        Ok(Ok(status)) => (status.code(), false),
        Ok(Err(e)) => {
            return Err(format!("等待认证适配器退出失败: {}", e));
        }
        Err(_) => {
            // 超时：kill 进程
            let _ = child.kill().await;
            (None, true)
        }
    };

    let stdout_bytes = stdout_task.await.unwrap_or_default();
    let stderr_bytes = stderr_task.await.unwrap_or_default();

    // 11. 截断输出到 1MB（复用 spawn::truncate_output）
    let stdout = truncate_output(&stdout_bytes);
    let stderr = truncate_output(&stderr_bytes);

    let duration_ms = start.elapsed().as_millis() as u64;

    log_debug!(
        "[Frp Sandbox] 认证适配器完成: provider={}, exit_code={:?}, 耗时={}ms, 超时={}",
        provider_id,
        exit_code,
        duration_ms,
        timed_out
    );

    Ok(ProcessResult {
        exit_code,
        stdout,
        stderr,
        timed_out,
        duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::super::binary::host_matches;

    #[test]
    fn host_matches_wildcard() {
        // 通配符：匹配任意子域名
        assert!(host_matches("jp-4.qwq.fan", "*.qwq.fan"));
        assert!(host_matches("us-1.qwq.fan", "*.qwq.fan"));
        // 通配符不匹配裸域名（需至少一层子域名）
        assert!(!host_matches("qwq.fan", "*.qwq.fan"));
        // 不匹配其他域名
        assert!(!host_matches("evil.example.com", "*.qwq.fan"));
    }

    #[test]
    fn host_matches_exact() {
        assert!(host_matches("frps.acme.example.com", "frps.acme.example.com"));
        assert!(!host_matches("other.acme.example.com", "frps.acme.example.com"));
        // host:port 项比较时调用方先剥离端口，这里验证纯 host 匹配
        assert!(!host_matches("frps.acme.example.com:7000", "frps.acme.example.com"));
    }

    #[test]
    fn host_matches_whitelist_forms() {
        // 模拟 validate_network_permissions 的白名单匹配逻辑：
        // 完整 host:port 匹配、host 匹配、通配符匹配三种形式
        let server_addr = "jp-4.qwq.fan";
        let addr_host = server_addr.split(':').next().unwrap_or(server_addr);
        let allowed = ["*.qwq.fan"];

        let matched = allowed.iter().any(|s| {
            let s = s.trim();
            if s == server_addr {
                return true;
            }
            let s_host = s.split(':').next().unwrap_or(s);
            s_host == addr_host || host_matches(addr_host, s_host)
        });
        assert!(matched);
    }
}
