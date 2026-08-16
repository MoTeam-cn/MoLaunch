//! easytier / Scaffolding 联机 IPC 动作。
//!
//! - `easytier_join`：拉起 easytier-core 加入虚拟网络（房主固定 IP，房客 DHCP）
//! - `easytier_stop`：停止当前 easytier 子进程
//! - `scaffolding_host_start`：房主一站式启动（探测 MC 端口 → 联机中心 → easytier）
//! - `scaffolding_host_stop`：停止联机中心与 easytier
//! - `scaffolding_client_probe`：房客解析房间码 → 加入网络 → 探测房主 MC 服务

use serde::{Deserialize, Serialize};

use crate::handler;
use crate::log_error;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::online::scaffolding::client as scaffolding_client;
use crate::minecraft::online::scaffolding::code as room_code;
use crate::minecraft::online::scaffolding::easytier::{EasyTier, HOST_VIRTUAL_IP};
use crate::minecraft::online::scaffolding::server::{ScaffoldingServer, ScaffoldingServerState};
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{Emitter, Manager};

/// 房客侧默认 hostname（未配置 network_identity 时使用）
const DEFAULT_GUEST_HOSTNAME: &str = "mo-launch-guest";

/// easytier 运行状态推送事件（前端经 useTauriEvent 监听）
const EASYTIER_STATUS_EVENT: &str = "easytier-status";

/// 房主自动关闭房间事件（后端→房主前端，触发房间清理登记）
const HOST_AUTO_CLOSE_EVENT: &str = "scaffolding-host-auto-close";

/// 房主 MC 端口变更事件（后端→房主前端，展示实时端口）
const MC_PORT_CHANGE_EVENT: &str = "scaffolding-mc-port-change";

/// 房主后台监视循环周期（5s）
const WATCH_INTERVAL: Duration = Duration::from_secs(5);

/// 端口连续不可达次数阈值（6 次 = 30s，触发自动关房）
const AUTO_CLOSE_FAIL_LIMIT: u32 = 6;

/// `EasytierJoinParams.no_tun` 缺省值：不创建虚拟网卡（用户态转发，无需管理员权限）
fn default_no_tun() -> bool {
    true
}

/// 构造 easytier 运行状态 payload（`easytier-status` 事件推送 / `easytier_status` IPC 查询共用）
fn easytier_status_payload(easytier: &Option<EasyTier>) -> serde_json::Value {
    match easytier {
        Some(e) => serde_json::json!({
            "joined": true,
            "version": e.version(),
            "pid": e.pid(),
            "rpcPortal": e.rpc_portal(),
            "networkName": e.network_name(),
            "virtualIp": e.virtual_ip().unwrap_or(""),
        }),
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
    let payload = easytier_status_payload(&guard);
    let _ = app.emit(EASYTIER_STATUS_EVENT, payload);
}

/// 解析 easytier-core 可执行文件路径。
///
/// 优先级：配置的绝对路径（直接校验存在性）→ 相对路径兼容旧配置（依次尝试
/// resource_dir / exe 同目录）→ 兜底释放内置嵌入式资源到 AppData/.Molaunch/easytier/。
fn resolve_core_path(app: &tauri::AppHandle, configured: &str) -> Result<PathBuf, String> {
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
    // 兜底：释放内置嵌入式资源（默认配置为空时走此路径）
    crate::resources::extract_easytier_core()
        .map_err(|e| format!("释放内置 easytier-core 失败: {e}"))
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

/// 读取配置中的公共 easytier 中继节点，展开为 `--peers` 参数序列
async fn configured_easytier_peers(state: &AppState) -> Vec<String> {
    let peers = &state.config.lock().await.online.easytier_public_peers;
    let mut args = Vec::new();
    for p in peers.iter().filter(|p| !p.trim().is_empty()) {
        args.push("--peers".to_string());
        args.push(p.trim().to_string());
    }
    args
}

/// 按当前游戏进程 PID 探测 MC 局域网端口（沿用 lan_probe 逻辑）
async fn probe_mc_port(state: &AppState) -> Option<u16> {
    let pid = *state.current_pid.lock().await;
    match pid {
        Some(pid) => crate::minecraft::launch::watcher::ports::listening_tcp_ports(pid)
            .into_iter()
            .next(),
        None => None,
    }
}

/// 房主后台监视循环：每 5s 扫描游戏监听端口并回写联机中心。
///
/// - 手动端口设置时跳过自动更新（最高权重），不自动关房；
/// - 从未探测到端口（游戏尚未开局域网）时无限等待（支持先开房后开局域网）；
///   仅「已探测到过端口后再不可达」连续 `AUTO_CLOSE_FAIL_LIMIT` 次（30s）
///   自动关闭房间并推送事件；
/// - 外部 `easytier_stop`/`scaffolding_host_stop` 抢先时（easytier 为 None）直接退出。
async fn host_watch_loop(
    center_state: ScaffoldingServerState,
    app: tauri::AppHandle,
    state: AppState,
) {
    let mut current_mc_port: Option<u16> = None;
    let mut fail_count: u32 = 0;
    loop {
        if state.easytier.lock().await.is_none() {
            return;
        }
        // 手动端口最高权重：始终同步手动值，不自动覆盖、不自动关房
        if let Some(manual) = *state.manual_mc_port.lock().await {
            if current_mc_port != Some(manual) {
                current_mc_port = Some(manual);
                center_state.set_mc_port(Some(manual));
                log_info!("[Online] 房主监视: 手动 MC 端口 {manual} 已同步");
            }
            tokio::time::sleep(WATCH_INTERVAL).await;
            continue;
        }
        let pid = *state.current_pid.lock().await;
        let ports = pid
            .map(crate::minecraft::launch::watcher::ports::listening_tcp_ports)
            .unwrap_or_default();
        if ports.is_empty() {
            // 从未探测到端口（游戏尚未开局域网）→ 无限等待，不累计失败，
            // 支持「先开房后开局域网」场景；仅「已探测到过端口后再不可达」触发自动关房
            if current_mc_port.is_some() {
                fail_count += 1;
                if fail_count >= AUTO_CLOSE_FAIL_LIMIT {
                    log_warn!("[Online] 房主监视: MC 端口连续 {fail_count} 次不可达，自动关闭房间");
                    auto_close_room(&state, &app).await;
                    return;
                }
            }
        } else {
            fail_count = 0;
            // 已知端口仍在监听则保持不变；否则取与已知端口最接近者（无已知时取升序第一个）
            let chosen = match current_mc_port {
                Some(cur) if ports.contains(&cur) => cur,
                _ => {
                    let target = current_mc_port.unwrap_or(0);
                    *ports
                        .iter()
                        .min_by_key(|p| p.abs_diff(target))
                        .unwrap_or(&ports[0])
                }
            };
            if current_mc_port != Some(chosen) {
                current_mc_port = Some(chosen);
                center_state.set_mc_port(Some(chosen));
                let _ = app.emit(
                    MC_PORT_CHANGE_EVENT,
                    serde_json::json!({ "mcPort": chosen }),
                );
                log_info!("[Online] 房主监视: MC 端口已更新为 {chosen}");
            }
        }
        tokio::time::sleep(WATCH_INTERVAL).await;
    }
}

/// 自动关闭房间：停 easytier + 停联机中心 + 清空状态 + 推送前端事件。
///
/// 幂等：各状态为 None 时 no-op，重复调用安全。
async fn auto_close_room(state: &AppState, app: &tauri::AppHandle) {
    let old_easytier = state.easytier.lock().await.take();
    if let Some(old_easytier) = old_easytier {
        old_easytier.stop().await;
    }
    let old_server = state.scaffolding_server.lock().await.take();
    if let Some(old_server) = old_server {
        old_server.stop().await;
    }
    *state.scaffolding_host_watch.lock().await = None;
    *state.manual_mc_port.lock().await = None;
    emit_easytier_status(app, state).await;
    let _ = app.emit(
        HOST_AUTO_CLOSE_EVENT,
        serde_json::json!({ "reason": "mc_unreachable" }),
    );
    log_info!("[Online] 房间已自动关闭（MC 服务 30s 不可达）");
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

/// `scaffolding_host_start` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingHostStartParams {
    /// 完整房间码 `U/NNNN-NNNN-SSSS-SSSS`（解析网络名与密钥）
    pub room_code: String,
    /// 房主 MC 局域网端口（缺省时按游戏进程探测）
    #[serde(default)]
    pub mc_port: Option<u16>,
}

/// `scaffolding_client_probe` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingClientProbeParams {
    /// 完整房间码 `U/NNNN-NNNN-SSSS-SSSS`
    pub room_code: String,
    /// 联机中心虚拟 IP（缺省取房主固定 `10.144.144.1`）
    #[serde(default)]
    pub center_ip: Option<String>,
    /// 联机中心 TCP 端口（缺省 13448）
    #[serde(default)]
    pub center_port: Option<u16>,
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

/// `scaffolding_host_start` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingHostStartResponse {
    pub success: bool,
    /// 联机中心实际监听端口
    pub center_port: u16,
    /// 中心 hostname（`scaffolding-mc-server-{center_port}`）
    pub hostname: String,
    /// 房主 MC 局域网端口（先开房后开局域网时为 None，后台监视发现端口后自动更新）
    pub mc_port: Option<u16>,
    /// rpc-portal 地址
    pub rpc_portal: String,
    /// easytier 子进程 PID
    pub pid: Option<u32>,
}

/// `scaffolding_host_set_mc_port` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingHostSetMcPortParams {
    /// 手动指定的 MC 端口（None 清除手动覆盖，恢复自动探测）
    #[serde(default)]
    pub mc_port: Option<u16>,
}

/// `scaffolding_client_probe` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldingClientProbeResponse {
    pub success: bool,
    /// 房主虚拟 IP（MC 客户端连接目标，配合 lan_fake 转发）
    pub mc_ip: String,
    /// 房主 MC 局域网端口
    pub mc_port: u16,
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

            let core_path = resolve_core_path(&app, &configured_core_path(&state).await)?;
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
                old.stop().await;
                log_info!("[Online] easytier 已停止");
            }
            emit_easytier_status(&app, &state).await;
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "scaffolding_host_start",
        handler!(state, app, params, {
            let p: ScaffoldingHostStartParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            let (network_name, network_secret) = room_code::parse(&p.room_code)?;

            // 停止旧实例（后台监视 + easytier + 联机中心），保证幂等
            let old_watch = state.scaffolding_host_watch.lock().await.take();
            if let Some(old_watch) = old_watch {
                old_watch.abort();
            }
            *state.manual_mc_port.lock().await = None;
            let old_easytier = state.easytier.lock().await.take();
            if let Some(old_easytier) = old_easytier {
                old_easytier.stop().await;
            }
            let old_server = state.scaffolding_server.lock().await.take();
            if let Some(old_server) = old_server {
                old_server.stop().await;
            }

            // MC 端口：显式参数优先，其次按游戏进程探测；均未获知时允许先开房，
            // 由后台监视循环发现端口后自动回写（host_watch_loop）
            let mc_port = match p.mc_port {
                Some(port) => Some(port),
                None => probe_mc_port(&state).await,
            };

            // 启动联机中心（监听虚拟 IP，MC 端口写入共享状态）
            let center_state = ScaffoldingServerState::new();
            center_state.set_mc_port(mc_port);
            let server = ScaffoldingServer::start_on(center_state.clone()).await?;
            let hostname = server.hostname();
            let center_port = server.port();

            // 启动 easytier（房主固定虚拟 IP + 中心 hostname；no-tun 迁移中暂保持 TUN）
            let core_path = resolve_core_path(&app, &configured_core_path(&state).await)?;
            let cli_path = resolve_cli_path(&core_path);
            let mut extra = Vec::new();
            extra.extend(configured_easytier_peers(&state).await);
            let easytier = match EasyTier::join(
                &core_path,
                &cli_path,
                &network_name,
                &network_secret,
                Some(HOST_VIRTUAL_IP),
                &hostname,
                extra,
                false,
            )
            .await
            {
                Ok(e) => e,
                Err(e) => {
                    let _ = server.stop().await;
                    return Err(e);
                }
            };
            let rpc_portal = easytier.rpc_portal().to_string();
            let pid = easytier.pid();
            *state.scaffolding_server.lock().await = Some(server);
            *state.easytier.lock().await = Some(easytier);
            emit_easytier_status(&app, &state).await;

            // 启动后台监视循环（端口热更新 + 自动关房）
            let watch_center = center_state.clone();
            let watch_app = app.clone();
            let watch_state = state.clone();
            let watch = tokio::spawn(async move {
                host_watch_loop(watch_center, watch_app, watch_state).await;
            });
            *state.scaffolding_host_watch.lock().await = Some(watch.abort_handle());

            log_info!(
                "[Online] 房主联机中心已启动: center_port={}, hostname={}, mc_port={:?}",
                center_port,
                hostname,
                mc_port
            );
            serde_json::to_value(ScaffoldingHostStartResponse {
                success: true,
                center_port,
                hostname,
                mc_port,
                rpc_portal,
                pid,
            })
            .map_err(|e| e.to_string())
        }),
    );

    d.register(
        "scaffolding_host_stop",
        handler!(state, app, _params, {
            // 中止后台监视任务，清除手动端口覆盖
            let watch = state.scaffolding_host_watch.lock().await.take();
            if let Some(watch) = watch {
                watch.abort();
            }
            *state.manual_mc_port.lock().await = None;
            let old_easytier = state.easytier.lock().await.take();
            if let Some(old_easytier) = old_easytier {
                old_easytier.stop().await;
            }
            let old_server = state.scaffolding_server.lock().await.take();
            if let Some(old_server) = old_server {
                old_server.stop().await;
            }
            emit_easytier_status(&app, &state).await;
            log_info!("[Online] 房主联机中心已停止");
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "scaffolding_client_probe",
        handler!(state, app, params, {
            let p: ScaffoldingClientProbeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            let (network_name, network_secret) = room_code::parse(&p.room_code)?;

            // 若尚未加入网络，先以房客身份（--dhcp）加入；no-tun 迁移中暂保持 TUN
            if state.easytier.lock().await.is_none() {
                let core_path = resolve_core_path(&app, &configured_core_path(&state).await)?;
                let cli_path = resolve_cli_path(&core_path);
                let hostname = {
                    let identity = configured_network_identity(&state).await;
                    if identity.is_empty() {
                        DEFAULT_GUEST_HOSTNAME.to_string()
                    } else {
                        identity
                    }
                };
                let mut extra = Vec::new();
                extra.extend(configured_easytier_peers(&state).await);
                let easytier = EasyTier::join(
                    &core_path,
                    &cli_path,
                    &network_name,
                    &network_secret,
                    None,
                    &hostname,
                    extra,
                    false,
                )
                .await?;
                log_info!(
                    "[Online] 房客已加入网络 {}（hostname={}）",
                    network_name,
                    hostname
                );
                *state.easytier.lock().await = Some(easytier);
                emit_easytier_status(&app, &state).await;
            }

            // 解析联机中心地址：显式参数优先，否则经 easytier-cli 从虚拟网络自动发现
            let (center_ip, center_port) = {
                let guard = state.easytier.lock().await;
                let easytier = guard
                    .as_ref()
                    .ok_or_else(|| "easytier 未加入网络".to_string())?;
                scaffolding_client::resolve_center_addr(
                    p.center_ip.as_deref(),
                    p.center_port,
                    easytier,
                )
                .await?
            };
            match scaffolding_client::discover_mc(&center_ip, center_port).await {
                Ok((mc_ip, mc_port)) => {
                    log_info!("[Online] 房客发现房主 MC 服务: {}:{}", mc_ip, mc_port);
                    serde_json::to_value(ScaffoldingClientProbeResponse {
                        success: true,
                        mc_ip,
                        mc_port,
                    })
                    .map_err(|e| e.to_string())
                }
                Err(e) => {
                    log_error!("[Online] 房客探测联机中心失败: {e}");
                    Err(e)
                }
            }
        }),
    );

    d.register(
        "scaffolding_host_set_mc_port",
        handler!(state, _app, params, {
            let p: ScaffoldingHostSetMcPortParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            *state.manual_mc_port.lock().await = p.mc_port;
            // 手动端口立即生效（联机中心直接回写）；清除时由监视循环自动补回
            if let Some(server) = state.scaffolding_server.lock().await.as_ref() {
                server.state().set_mc_port(p.mc_port);
            }
            log_info!("[Online] 房主手动 MC 端口: {:?}", p.mc_port);
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "scaffolding_client_poll",
        handler!(state, _app, params, {
            let _p: ScaffoldingClientProbeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            // 轻量轮询：不 join，仅解析中心地址后探测 MC 端口（复用 probe 发现逻辑）
            let (center_ip, center_port) = {
                let guard = state.easytier.lock().await;
                let easytier = guard
                    .as_ref()
                    .ok_or_else(|| "easytier 未加入网络".to_string())?;
                scaffolding_client::resolve_center_addr(None, None, easytier).await?
            };
            match scaffolding_client::discover_mc(&center_ip, center_port).await {
                Ok((mc_ip, mc_port)) => serde_json::to_value(ScaffoldingClientProbeResponse {
                    success: true,
                    mc_ip,
                    mc_port,
                })
                .map_err(|e| e.to_string()),
                Err(e) => {
                    log_error!("[Online] 房客轮询联机中心失败: {e}");
                    Err(e)
                }
            }
        }),
    );

    d.register(
        "easytier_status",
        handler!(state, _app, _params, {
            let guard = state.easytier.lock().await;
            serde_json::to_value(easytier_status_payload(&guard)).map_err(|e| e.to_string())
        }),
    );
}
