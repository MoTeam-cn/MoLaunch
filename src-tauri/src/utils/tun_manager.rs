//! TUN 桥接管理 action（阶段三子任务 5：数据分发打通）
//!
//! 由 `online_manager::DISPATCHER` 调用 `register_tun_actions` 注册 3 个 IPC action：
//! - `tun_start`：创建 TUN 接口 + 启动读写循环 + 开始向前端 emit `online://tun-packet-out` 事件
//! - `tun_forward_to`：前端 DataChannel 收到消息后调用，解码协议帧并写入 TUN
//! - `tun_stop`：停止桥接，销毁 TUN 接口
//!
//! # 数据流
//!
//! ```text
//! 后端 TUN 读包               前端 DataChannel
//!   │                            │
//!   │ 1. TUN.recv → IP 包        │
//!   │ 2. protocol::encode → 帧   │
//!   │ 3. emit(EVENT_TUN_PACKET_OUT, frame)
//!   │ ──────────────────────>    │ 4. listen → broadcastPacket / channel.send
//!   │                            │
//!   │                            │ 5. DataChannel.onmessage → ArrayBuffer
//!   │ <──────────────────────    │ 6. invoke('online_manager', {action:'tun_forward_to', params:{message: base64}})
//!   │ 7. base64 decode → frame   │
//!   │ 8. protocol::decode → IP   │
//!   │ 9. TUN.send → 写入接口     │
//! ```
//!
//! # 二进制传输约定
//!
//! Tauri IPC 走 JSON 序列化，二进制数据用 base64 字符串传递（`message_base64` 字段）。
//! IP 包 MTU 1400 字节，base64 后约 1870 字节，开销可接受。

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::online::bridge::VirtualLanBridge;
use crate::utils::dispatcher::Dispatcher;

// ============================================================
// 参数 / 返回类型
// ============================================================

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

// ============================================================
// 注册入口
// ============================================================

/// 注册全部 TUN 管理 action 到 dispatcher
pub fn register_tun_actions(d: &mut Dispatcher) {
    register_tun_start(d);
    register_tun_forward_to(d);
    register_tun_stop(d);
}

// ============================================================
// 各 action 注册
// ============================================================

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
        let bridge = VirtualLanBridge::start(&p.ipv4, p.prefix_len, app.clone()).await
            .map_err(|e| {
                log_error!("[Online] tun_start 失败: {}", e);
                e.to_string()
            })?;

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

        // 取当前 bridge
        let bridge_opt = state.virtual_lan_bridge.lock().await;
        let bridge = match bridge_opt.as_ref() {
            Some(b) => b,
            None => {
                log_warn!("[Online] tun_forward_to: bridge 未启动，忽略消息");
                return Err("TUN bridge 未启动".to_string());
            }
        };

        // 调用 bridge.forward_from_datachannel（不能跨 await 持有 Mutex 守卫，先 clone 引用）
        // VirtualLanBridge 内部 write_tx 是 Sender，可安全跨 await
        let result = bridge.forward_from_datachannel(&raw).await
            .map_err(|e| {
                log_warn!("[Online] tun_forward_to 失败: {}", e);
                e
            })?;

        let (is_data, packet_len) = match &result {
            Some(packet) => (true, packet.len()),
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
