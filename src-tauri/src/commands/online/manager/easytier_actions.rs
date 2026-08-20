//! easytier / Scaffolding 联机 IPC 动作。
//!
//! - `easytier_join`：拉起 easytier-core 加入虚拟网络（房主固定 IP，房客 DHCP）
//! - `easytier_stop`：停止当前 easytier 子进程（清空房客 port-forward 记录）
//! - `scaffolding_host_start`：房主一站式启动（探测 MC 端口 → 联机中心 → easytier no-tun + 白名单）
//! - `scaffolding_host_stop`：停止联机中心与 easytier
//! - `scaffolding_client_probe`：房客解析房间码 → 加入网络（no-tun）→ 建立 port-forward → 探测房主 MC 服务

use serde::{Deserialize, Serialize};

use crate::handler;
use crate::log_info;
use crate::minecraft::online::scaffolding::easytier::{EasyTier, HOST_VIRTUAL_IP};
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;
use std::path::{Path, PathBuf};
use tauri::Emitter;
use tauri::Manager;

mod guest;
mod host;
mod watch;

/// 房客侧默认 hostname（未配置 network_identity 时使用）
const DEFAULT_GUEST_HOSTNAME: &str = "mo-launch-guest";

/// easytier 运行状态推送事件（前端经 useTauriEvent 监听）
const EASYTIER_STATUS_EVENT: &str = "easytier-status";

/// `EasytierJoinParams.no_tun` 缺省值：不创建虚拟网卡（用户态转发，无需管理员权限）
fn default_no_tun() -> bool {
    true
}

/// 构造 easytier 运行状态 payload（`easytier-status` 事件推送 / `easytier_status` IPC 查询共用）
///
/// 房客 DHCP 模式经 `query_virtual_ip` 查询实际分配地址（no-tun 下无虚拟网卡，须经 CLI 回显）。
async fn easytier_status_payload(easytier: &Option<EasyTier>) -> serde_json::Value {
    match easytier {
        Some(e) => {
            let virtual_ip = e.query_virtual_ip().await.unwrap_or_default();
            serde_json::json!({
                "joined": true,
                "version": e.version(),
                "pid": e.pid(),
                "rpcPortal": e.rpc_portal(),
                "networkName": e.network_name(),
                "virtualIp": virtual_ip,
            })
        }
        None => serde_json::json!({
            "joined": false,
            "version": "",
            "pid": null,
            "rpcPortal": "",
            "networkName": "",
            "virtualIp": "",
        }),
    }
}

/// 构造并推送 easytier 运行状态事件（加入/停止后调用，供设备页展示）
async fn emit_easytier_status(app: &tauri::AppHandle, state: &AppState) {
    let guard = state.easytier.lock().await;
    let payload = easytier_status_payload(&guard).await;
    let _ = app.emit(EASYTIER_STATUS_EVENT, payload);
}

/// 解析 easytier-core 可执行文件路径。
///
/// 优先级：配置的绝对路径（直接校验存在性）→ 相对路径兼容旧配置（依次尝试
/// resource_dir / exe 同目录）→ 兜底自动下载安装到 AppData/.Molaunch/easytier/。
async fn resolve_core_path(
    state: &AppState,
    app: &tauri::AppHandle,
    configured: &str,
) -> Result<PathBuf, String> {
    let p = PathBuf::from(configured);
    if p.is_absolute() {
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "easytier-core 不存在: {}（请在联机设置中指定正确路径）",
            p.display()
        ));
    }
    // 相对路径（旧配置兼容，如 `sidecar/easytier-core.exe`）
    if !configured.is_empty() {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(dir) = app.path().resource_dir() {
            candidates.push(dir.join(&p));
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.join(&p));
            }
        }
        for c in &candidates {
            if c.is_file() {
                return Ok(c.clone());
            }
        }
    }
    // 兜底：自动下载安装（默认配置为空时走此路径，已安装直接返回）
    crate::commands::online::manager::easytier_install::ensure_installed(state, app)
        .await
        .map_err(|e| format!("安装 easytier-core 失败: {e}"))
}

/// 解析 easytier-cli 可执行文件路径（与 core 同目录，随包附带）。
///
/// 自定义 core 路径时按同目录推断 cli；内置资源释放时两者同目录释放。
fn resolve_cli_path(core_path: &Path) -> PathBuf {
    let cli_name = if cfg!(target_os = "windows") {
        "easytier-cli.exe"
    } else {
        "easytier-cli"
    };
    core_path.with_file_name(cli_name)
}

/// 读取配置中的 easytier-core 路径
async fn configured_core_path(state: &AppState) -> String {
    state.config.lock().await.online.easytier_core_path.clone()
}

/// 读取配置中的虚拟网络节点标识（房客 hostname 用）
async fn configured_network_identity(state: &AppState) -> String {
    state.config.lock().await.online.network_identity.clone()
}

/// 项目自建 easytier 信令节点（默认内置，前端设置页不展示）：保证组网必有可用信令节点
const DEFAULT_SIGNALING_PEER: &str = "wss://node1.molaunch.moiu.cn";

/// 读取配置中的公共 easytier 节点，展开为 `--peers` 参数序列。
///
/// 配置缺失或用户未配置时兜底注入默认信令节点，保证 `--peers` 永远非空。
async fn configured_easytier_peers(state: &AppState) -> Vec<String> {
    let peers = &state.config.lock().await.online.easytier_public_peers;
    let mut list: Vec<String> = peers
        .iter()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if !list.iter().any(|p| p == DEFAULT_SIGNALING_PEER) {
        list.push(DEFAULT_SIGNALING_PEER.to_string());
    }
    let mut args = Vec::new();
    for p in list {
        args.push("--peers".to_string());
        args.push(p);
    }
    args
}

/// `easytier_join` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EasytierJoinParams {
    /// 虚拟网络名（房主与房客需一致，来自房间码 N 段）
    pub network_name: String,
    /// 虚拟网络密钥（来自房间码 S 段）
    pub network_secret: String,
    /// 是否为房主（房主固定虚拟 IP `10.144.144.1`，房客走 `--dhcp`）
    #[serde(default)]
    pub is_host: bool,
    /// 是否 no-tun 模式（默认 true：不创建虚拟网卡，走用户态端口转发）
    #[serde(default = "default_no_tun")]
    pub no_tun: bool,
    /// 节点 hostname（房主必须为 `scaffolding-mc-server-{center_port}`，联机中心端口）
    #[serde(default)]
    pub hostname: Option<String>,
    /// 追加 easytier-core CLI 参数（如 `--peers` 公共服务器）
    #[serde(default)]
    pub extra: Option<Vec<String>>,
}

/// `easytier_join` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EasytierJoinResponse {
    pub success: bool,
    /// rpc-portal 地址（`127.0.0.1:动态端口`，供后续 easytier CLI 查询）
    pub rpc_portal: String,
    /// 子进程 PID
    pub pid: Option<u32>,
}

/// 注册 easytier / Scaffolding 动作到 dispatcher
pub fn register(d: &mut Dispatcher) {
    d.register(
        "easytier_join",
        handler!(state, app, params, {
            let p: EasytierJoinParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;

            // 若已有实例先停止（防泄漏）
            let old = state.easytier.lock().await.take();
            if let Some(old) = old {
                old.stop().await;
            }

            let core_path =
                resolve_core_path(&state, &app, &configured_core_path(&state).await).await?;
            let cli_path = resolve_cli_path(&core_path);
            let ip = if p.is_host {
                Some(HOST_VIRTUAL_IP)
            } else {
                None
            };
            let guest_hostname = {
                let identity = configured_network_identity(&state).await;
                if identity.is_empty() {
                    DEFAULT_GUEST_HOSTNAME.to_string()
                } else {
                    identity
                }
            };
            let hostname = match (p.is_host, p.hostname) {
                (true, Some(h)) => h,
                (true, None) => {
                    return Err(
                        "房主模式必须提供 hostname（格式 scaffolding-mc-server-{center_port}）"
                            .to_string(),
                    )
                }
                (false, h) => h.unwrap_or(guest_hostname),
            };

            let mut extra = p.extra.unwrap_or_default();
            extra.extend(configured_easytier_peers(&state).await);
            let easytier = EasyTier::join(
                &core_path,
                &cli_path,
                &p.network_name,
                &p.network_secret,
                ip,
                &hostname,
                extra,
                p.no_tun,
            )
            .await?;
            let rpc_portal = easytier.rpc_portal().to_string();
            let pid = easytier.pid();
            *state.easytier.lock().await = Some(easytier);
            emit_easytier_status(&app, &state).await;

            log_info!(
                "[Online] easytier 已加入网络 {}（hostname={}，rpc={}）",
                p.network_name,
                hostname,
                rpc_portal
            );
            serde_json::to_value(EasytierJoinResponse {
                success: true,
                rpc_portal,
                pid,
            })
            .map_err(|e| e.to_string())
        }),
    );

    d.register(
        "easytier_stop",
        handler!(state, app, _params, {
            let old = state.easytier.lock().await.take();
            if let Some(old) = old {
                // 先经 RPC 移除房客 port-forward 规则，再停止进程
                let rules = state.client_port_forwards.lock().await;
                for rule in rules.iter() {
                    let _ = old.remove_port_forward(&rule.proto, &rule.bind_addr).await;
                }
                drop(rules);
                old.stop().await;
                *state.client_port_forwards.lock().await = Vec::new();
                log_info!("[Online] easytier 已停止");
            }
            emit_easytier_status(&app, &state).await;
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "easytier_status",
        handler!(state, _app, _params, {
            let guard = state.easytier.lock().await;
            serde_json::to_value(easytier_status_payload(&guard).await).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "easytier_peers",
        handler!(state, _app, _params, {
            let guard = state.easytier.lock().await;
            match &*guard {
                Some(et) => serde_json::to_value(et.peers().await?).map_err(|e| e.to_string()),
                None => Ok(serde_json::json!([])),
            }
        }),
    );

    host::register_host(d);
    guest::register_guest(d);
}
