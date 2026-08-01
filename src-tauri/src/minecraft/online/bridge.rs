//! DataChannel ↔ TUN 桥接（阶段三子任务 3）
//!
//! 协调 TUN 接口读写循环与前端 DataChannel 的事件转发。
//! 后端 TUN 读包 → Tauri 事件 → 前端 DataChannel.send；
//! 前端 DataChannel.onmessage → IPC 命令 → 写入 TUN。

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::minecraft::online::protocol::{self, Message};
use crate::minecraft::online::tun::VirtualNet;
use crate::{log_debug, log_error, log_info, log_warn};

/// 桥接事件名：后端从 TUN 读到包，发给前端通过 DataChannel 发送
pub const EVENT_TUN_PACKET_OUT: &str = "online://tun-packet-out";

/// 桥接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeState {
    /// 未启动
    Stopped,
    /// 运行中
    Running,
    /// 已停止（错误或主动停止）
    Closed,
}

/// 虚拟局域网桥接器
///
/// 持有 TUN 接口写通道、读写循环 task 句柄与桥接状态。
/// 房主与加入方共用此结构，区别仅在于 TUN 接口的虚拟 IP 分配。
pub struct VirtualLanBridge {
    /// 写入 TUN 的发送端（前端 DataChannel 收到的包通过此通道写入 TUN）
    write_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    /// 读写循环 task 句柄（合并 select! 单循环，abort 可停止）
    handle: tokio::task::JoinHandle<()>,
    /// 桥接状态
    state: Arc<Mutex<BridgeState>>,
}

impl VirtualLanBridge {
    /// 启动桥接（房主与加入方通用）
    ///
    /// # 参数
    ///
    /// - `ipv4`：虚拟 IP（如 `10.244.1.1`）
    /// - `prefix_len`：子网前缀长度（如 24）
    /// - `app_handle`：Tauri AppHandle，用于发送事件给前端
    pub async fn start(
        ipv4: &str,
        prefix_len: u8,
        app_handle: tauri::AppHandle,
    ) -> tauri::Result<Self> {
        use tauri::Emitter;

        log_info!(
            "[Online] 启动 VirtualLanBridge: ipv4={}/{}, prefix_len={}",
            ipv4,
            prefix_len,
            prefix_len
        );

        // Windows 专属：从 AppData 释放并加载 wintun.dll
        // （编译时 include_bytes! 嵌入二进制，运行时释放到 %APPDATA%/.Molaunch/wintun.dll）
        #[cfg(windows)]
        let wintun_path: Option<std::path::PathBuf> = {
            match crate::resources::extract_wintun() {
                Ok(p) => {
                    log_debug!("[Online] wintun.dll 已释放: {}", p.display());
                    Some(p)
                }
                Err(e) => {
                    log_warn!("[Online] 释放 wintun.dll 失败: {}, 回退到默认 DLL 搜索", e);
                    None
                }
            }
        };
        #[cfg(not(windows))]
        let wintun_path: Option<std::path::PathBuf> = None;

        // 创建 TUN 接口
        let net = VirtualNet::create(ipv4, prefix_len, 1400, wintun_path.as_deref())
            .await
            .map_err(|e| {
                log_error!("[Online] TUN 接口创建失败: {}", e);
                tauri::Error::Anyhow(anyhow::anyhow!("TUN 接口创建失败: {}", e))
            })?;

        let info = net.info().clone();
        log_info!("[Online] TUN 接口已就绪: {:?}", info);

        // 启动读写循环 task：单循环 + select! 同时处理 TUN 读包和前端写包
        let (write_tx, mut write_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(128);
        let state = Arc::new(Mutex::new(BridgeState::Running));

        let device = net.into_device();

        // 启动读写循环 task
        let state_clone = state.clone();
        let app_handle_clone = app_handle.clone();
        let handle = tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];
            // 协议帧 seq 计数器：task 内部维护，外部无需读取
            let mut seq: u32 = 0;
            log_info!("[Online] TUN 读写循环启动: {}", info.name);

            loop {
                tokio::select! {
                    // 从 TUN 读包 → 编码 → 发事件给前端
                    read_result = device.recv(&mut buf) => {
                        match read_result {
                            Ok(len) if len > 0 => {
                                let packet = &buf[..len];
                                let msg = protocol::data_message(seq, packet);
                                seq = seq.wrapping_add(1);
                                let encoded = protocol::encode(&msg);

                                // 通过 Tauri 事件发给前端
                                if let Err(e) = app_handle_clone.emit(EVENT_TUN_PACKET_OUT, encoded) {
                                    log_warn!("[Online] TUN 包事件发送失败: {}", e);
                                }
                            }
                            Ok(_) => {
                                // len == 0，忽略
                            }
                            Err(e) => {
                                log_warn!("[Online] TUN 读错误: {}", e);
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            }
                        }
                    }
                    // 从前端 IPC 收到包 → 写入 TUN
                    write_result = write_rx.recv() => {
                        match write_result {
                            Some(packet) => {
                                if let Err(e) = device.send(&packet).await {
                                    log_warn!("[Online] TUN 写入失败: {}", e);
                                }
                            }
                            None => {
                                log_info!("[Online] TUN 写通道关闭，停止读写循环");
                                break;
                            }
                        }
                    }
                }
            }

            let mut st = state_clone.lock().await;
            *st = BridgeState::Closed;
            log_info!("[Online] TUN 读写循环退出: {}", info.name);
        });

        Ok(Self {
            write_tx,
            handle,
            state,
        })
    }

    /// 克隆 TUN 写通道发送端（供调用方在释放 Mutex 守卫后跨 await 发送）
    ///
    /// `mpsc::Sender` 是 Cloneable 的，clone 后可在释放 Mutex 守卫后安全跨 await 使用，
    /// 避免在 `tun_forward_to` 中跨 await 持有 `virtual_lan_bridge` Mutex 守卫导致死锁。
    pub fn write_tx_clone(&self) -> tokio::sync::mpsc::Sender<Vec<u8>> {
        self.write_tx.clone()
    }

    /// 解码 DataChannel 消息（同步，不写入 TUN）
    ///
    /// 前端从 DataChannel.onmessage 收到二进制消息后，
    /// 通过 IPC `online_manager` action `tun_forward_to` 调用。
    ///
    /// 调用方应在释放 Mutex 守卫后，将返回的 payload 通过 `write_tx_clone()` 获取的
    /// 发送端写入 TUN，避免跨 await 持有 Mutex 守卫。
    ///
    /// # 返回
    ///
    /// - `Ok(Some(packet))`：数据消息，返回 IP 包 payload（调用方需写入 TUN）
    /// - `Ok(None)`：控制/错误消息，非 IP 包，不写入 TUN
    /// - `Err`：协议帧解码失败
    pub fn decode_from_datachannel(raw_message: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let msg = protocol::decode(raw_message).map_err(|e| {
            log_warn!("[Online] DataChannel 消息解码失败: {}", e);
            e.to_string()
        })?;

        match msg {
            Message::Data { seq, payload } => {
                log_debug!(
                    "[Online] DataChannel → TUN: seq={}, len={}",
                    seq,
                    payload.len()
                );
                Ok(Some(payload))
            }
            Message::Control {
                seq,
                subtype,
                payload,
            } => {
                log_debug!(
                    "[Online] DataChannel 控制消息: seq={}, subtype={:?}, len={}",
                    seq,
                    subtype,
                    payload.len()
                );
                // 控制消息不写入 TUN
                Ok(None)
            }
            Message::Error { seq, message } => {
                log_warn!(
                    "[Online] DataChannel 错误消息: seq={}, msg={}",
                    seq,
                    message
                );
                Ok(None)
            }
        }
    }

    /// 停止桥接，销毁 TUN 接口
    pub async fn stop(&self) {
        log_info!("[Online] 停止 VirtualLanBridge");

        let mut st = self.state.lock().await;
        if *st == BridgeState::Stopped || *st == BridgeState::Closed {
            log_info!("[Online] Bridge 已停止，跳过");
            return;
        }

        self.handle.abort();
        *st = BridgeState::Closed;
    }

    /// 获取当前桥接状态
    pub async fn state(&self) -> BridgeState {
        *self.state.lock().await
    }
}

impl Drop for VirtualLanBridge {
    fn drop(&mut self) {
        // 确保读写循环 task 被终止
        self.handle.abort();
        log_debug!("[Online] VirtualLanBridge drop");
    }
}

#[cfg(test)]
#[path = "bridge_tests.rs"]
mod tests;
