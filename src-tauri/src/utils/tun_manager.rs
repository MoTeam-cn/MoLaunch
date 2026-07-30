//! TUN 桥接管理 action（阶段三子任务 5：数据分发打通）
//!
//! 由 `online_manager::DISPATCHER` 调用 `register_tun_actions` 注册 3 个 action：
//! `tun_start` / `tun_forward_to` / `tun_stop`。
//! 二进制数据通过 `message_base64` 字段以 base64 字符串经 Tauri IPC 传递。

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::online::bridge::VirtualLanBridge;
use crate::utils::dispatcher::Dispatcher;


/// `tun_start` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunStartParams {
    /// 虚拟 IPv4 地址（如 `10.244.1.1`）
    pub ipv4: String,
    /// 子网前缀长度（如 24，对应 `10.244.1.0/24`）
    pub prefix_len: u8,
}

/// `tun_start` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunStartResponse {
    /// TUN 接口名（如 `tun-molaunch`）
    pub interface_name: String,
    /// 虚拟 IP
    pub ipv4: String,
    /// 子网前缀长度
    pub prefix_len: u8,
    /// MTU
    pub mtu: u16,
}

/// `tun_forward_to` 参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunForwardParams {
    /// DataChannel 收到的二进制消息（协议帧编码后的字节），base64 编码字符串
    ///
    /// 前端从 `DataChannel.onmessage` 拿到 `ArrayBuffer` 后，
    /// 用 `btoa(String.fromCharCode(...new Uint8Array(buf)))` 转 base64 传入。
    pub message_base64: String,
}

/// `tun_forward_to` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunForwardResponse {
    /// 是否为数据包（true=已写入 TUN，false=控制/错误消息，未写入）
    pub is_data: bool,
    /// 解码出的 IP 包字节数（控制消息为 0）
    pub packet_len: usize,
}


/// 注册全部 TUN 管理 action 到 dispatcher
pub fn register_tun_actions(d: &mut Dispatcher) {
    register_tun_start(d);
    register_tun_forward_to(d);
    register_tun_stop(d);
    register_restart_as_admin(d);
}


fn register_tun_start(d: &mut Dispatcher) {
    d.register("tun_start", handler!(state, app, params, {
        let p: TunStartParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;

        log_info!(
            "[Online] tun_start: ipv4={}/{}, prefix_len={}",
            p.ipv4, p.prefix_len, p.prefix_len
        );

        // 若已有 bridge，先停止（防止泄漏）
        {
            let mut guard = state.virtual_lan_bridge.lock().await;
            if let Some(old) = guard.take() {
                log_warn!("[Online] tun_start: 检测到旧 bridge，先停止");
                old.stop().await;
            }
        }

        // 创建新 bridge
        let bridge = match VirtualLanBridge::start(&p.ipv4, p.prefix_len, app.clone()).await {
            Ok(b) => b,
            Err(e) => {
                let err_str = e.to_string();
                log_error!("[Online] tun_start 失败: {}", err_str);

                // 检测权限错误：Windows 上 wintun.dll 创建 TUN 接口需要管理员权限
                // os error 5 = ERROR_ACCESS_DENIED
                let is_permission_error = err_str.contains("os error 5")
                    || err_str.contains("拒绝访问")
                    || err_str.contains("Permission denied");

                if is_permission_error && !crate::minecraft::system::shell::is_admin() {
                    // 返回特殊错误标记，前端据此弹出管理员重启确认框
                    return Err(
                        "TUN_PERMISSION_DENIED:需要管理员权限来创建虚拟网卡，是否以管理员权限重启启动器？"
                            .to_string()
                    );
                }

                return Err(err_str);
            }
        };

        // 取接口元信息（在 bridge 启动后只能从外部记的入参里取，bridge 内部 info 已 move）
        // 这里直接用入参回传，TUN info 在 bridge 启动日志里已打印
        let response = TunStartResponse {
            interface_name: "tun-molaunch".to_string(),
            ipv4: p.ipv4.clone(),
            prefix_len: p.prefix_len,
            mtu: 1400,
        };

        let mut guard = state.virtual_lan_bridge.lock().await;
        *guard = Some(bridge);

        log_info!("[Online] tun_start 成功: {:?}", response);
        serde_json::to_value(response).map_err(|e| e.to_string())
    }));
}

fn register_tun_forward_to(d: &mut Dispatcher) {
    d.register("tun_forward_to", handler!(state, _app, params, {
        let p: TunForwardParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;

        // base64 解码
        let raw = base64::engine::general_purpose::STANDARD
            .decode(&p.message_base64)
            .map_err(|e| {
                log_warn!("[Online] tun_forward_to: base64 解码失败: {}", e);
                format!("base64 解码失败: {}", e)
            })?;

        // 取当前 bridge 的 write_tx clone（不持有 Mutex 守卫跨 await，防死锁）
        let write_tx = {
            let guard = state.virtual_lan_bridge.lock().await;
            match guard.as_ref() {
                Some(b) => b.write_tx_clone(),
                None => {
                    log_warn!("[Online] tun_forward_to: bridge 未启动，忽略消息");
                    return Err("TUN bridge 未启动".to_string());
                }
            }
        };
        // guard 已释放，write_tx 是 mpsc::Sender，可安全跨 await

        // 同步解码 DataChannel 消息（无需持有 bridge 引用）
        let decoded = VirtualLanBridge::decode_from_datachannel(&raw)
            .map_err(|e| {
                log_warn!("[Online] tun_forward_to 解码失败: {}", e);
                e
            })?;

        let (is_data, packet_len) = match &decoded {
            Some(packet) => {
                // 写入 TUN（跨 await，但不持有 Mutex 守卫）
                if write_tx.send(packet.clone()).await.is_err() {
                    log_warn!("[Online] tun_forward_to: TUN 写通道已关闭");
                    return Err("TUN 写通道已关闭".to_string());
                }
                (true, packet.len())
            }
            None => (false, 0),
        };

        log_debug!(
            "[Online] tun_forward_to: is_data={}, len={}",
            is_data, packet_len
        );

        let resp = TunForwardResponse { is_data, packet_len };
        serde_json::to_value(resp).map_err(|e| e.to_string())
    }));
}

fn register_tun_stop(d: &mut Dispatcher) {
    d.register("tun_stop", handler!(state, _app, _params, {
        log_info!("[Online] tun_stop");

        let mut guard = state.virtual_lan_bridge.lock().await;
        if let Some(bridge) = guard.take() {
            bridge.stop().await;
            log_info!("[Online] tun_stop 成功");
        } else {
            log_debug!("[Online] tun_stop: bridge 未启动，跳过");
        }

        serde_json::to_value(serde_json::json!({ "success": true }))
            .map_err(|e| e.to_string())
    }));
}

/// `restart_as_admin` action
///
/// 前端在 `tun_start` 返回 `TUN_PERMISSION_DENIED:` 错误并经用户确认后调用。
/// - release 模式：以管理员权限重启当前进程（ShellExecuteW "runas"），延迟 500ms 退出当前进程
/// - dev 模式：不自动重启（ShellExecuteW 启动的 exe 会丢失 cargo run 注入的 dev 环境变量，
///   导致无法连接 Vite dev server），返回 `dev_mode` 标记，前端提示用户用管理员权限终端
///   运行 `npm run tauri dev`
fn register_restart_as_admin(d: &mut Dispatcher) {
    d.register("restart_as_admin", handler!(_state, app, _params, {
        log_info!("[Online] restart_as_admin: 以管理员权限重启");

        // dev 模式：直接返回提示，不重启进程
        if cfg!(debug_assertions) {
            log_info!("[Online] restart_as_admin: dev 模式，跳过自动重启，提示用户用管理员终端启动");
            return serde_json::to_value(serde_json::json!({
                "success": false,
                "dev_mode": true,
                "message": "开发模式下无法自动重启，请用管理员权限的终端运行 npm run tauri dev",
            }))
                .map_err(|e| e.to_string());
        }

        crate::minecraft::system::shell::relaunch_as_admin(&[])?;

        // 延迟退出当前进程，给前端留时间收到 IPC 响应
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            log_info!("[Online] 退出当前进程（管理员重启）");
            app_clone.exit(0);
        });

        serde_json::to_value(serde_json::json!({ "success": true }))
            .map_err(|e| e.to_string())
    }));
}
