//! 虚拟网卡管理（阶段三 PoC）
//!
//! 跨平台 TUN 接口的创建、配置、读写与销毁。
//! 基于 `tun-rs` crate，Windows 底层为 Wintun 驱动，Linux/macOS 为系统原生 TUN。
//! Windows 需 `wintun.dll`，Linux/macOS 需 root 或 CAP_NET_ADMIN 权限。

use std::io;
use std::path::Path;
use tun_rs::{AsyncDevice, DeviceBuilder};

// 项目日志宏（logger.rs 中定义）
use crate::{log_info, log_warn};

/// 虚拟网卡接口元信息
#[derive(Debug, Clone)]
pub struct VirtualNetInfo {
    /// 接口名（如 `tun0` / `utun7` / `wintun0`）
    pub name: String,
    /// 虚拟 IPv4 地址（如 `10.244.1.1`）
    pub ipv4: String,
    /// 子网前缀长度（如 24）
    pub prefix_len: u8,
    /// MTU
    pub mtu: u16,
}

/// 虚拟网卡抽象
///
/// 持有 `AsyncDevice`（tokio 异步 TUN 接口），提供读写 IP 包的 API。
/// Drop 时自动关闭接口，无需手动 `close()`。
pub struct VirtualNet {
    device: AsyncDevice,
    info: VirtualNetInfo,
}

impl VirtualNet {
    /// 创建 TUN 接口并分配虚拟 IP
    ///
    /// # 参数
    ///
    /// - `ipv4`：虚拟 IPv4 地址（如 `10.244.1.1`）
    /// - `prefix_len`：子网前缀长度（如 24，对应 `10.244.1.0/24`）
    /// - `mtu`：MTU，默认 1400（避免 WebRTC DataChannel 分片）
    /// - `wintun_dll_path`：Windows 专属，显式指定 `wintun.dll` 路径（如 Tauri resources
    ///   解析的路径）；传 `None` 则回退到 Windows 默认 DLL 搜索（可执行文件同目录等）
    ///
    /// # 错误
    ///
    /// - Windows：缺少 `wintun.dll` 或无管理员权限
    /// - Linux/macOS：无 root 或 CAP_NET_ADMIN 权限
    pub async fn create(
        ipv4: &str,
        prefix_len: u8,
        mtu: u16,
        wintun_dll_path: Option<&Path>,
    ) -> io::Result<Self> {
        log_info!(
            "[Online] 创建 TUN 接口: ipv4={}/{}, mtu={}",
            ipv4,
            prefix_len,
            mtu
        );

        let builder = DeviceBuilder::new().ipv4(ipv4, prefix_len, None).mtu(mtu);

        // Windows 专属：显式指定 wintun.dll 路径，避免依赖默认 DLL 搜索顺序
        #[cfg(windows)]
        let builder = {
            use crate::log_debug;
            if let Some(path) = wintun_dll_path {
                let path_str = path.to_string_lossy().into_owned();
                log_debug!("[Online] 使用 wintun.dll 路径: {}", path_str);
                builder.with(|b| {
                    b.wintun_file(path_str.clone());
                })
            } else {
                builder
            }
        };

        // 非 Windows 平台忽略 wintun_dll_path 参数
        #[cfg(not(windows))]
        {
            let _ = wintun_dll_path;
        }

        let device = builder.build_async()?;

        let info = VirtualNetInfo {
            name: "tun-molaunch".to_string(),
            ipv4: ipv4.to_string(),
            prefix_len,
            mtu,
        };

        log_info!("[Online] TUN 接口已创建: {:?}", info);
        Ok(Self { device, info })
    }

    /// 获取接口元信息
    pub fn info(&self) -> &VirtualNetInfo {
        &self.info
    }

    /// 消费 VirtualNet，返回底层 AsyncDevice（供 bridge 直接操作）
    ///
    /// VirtualNet 不实现 Drop，调用方负责在不再使用时关闭 AsyncDevice。
    pub fn into_device(self) -> AsyncDevice {
        self.device
    }

    /// 从 TUN 接口读出一个 IP 包
    ///
    /// 返回读到的字节数与数据缓冲区。缓冲区大小建议 65535（最大 IP 包）。
    pub async fn recv_packet(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.device.recv(buf).await
    }

    /// 向 TUN 接口写入一个 IP 包
    ///
    /// 返回写入的字节数。
    pub async fn send_packet(&mut self, packet: &[u8]) -> io::Result<usize> {
        self.device.send(packet).await
    }

    /// 显式关闭接口（通常不需要，AsyncDevice Drop 时自动关闭）
    pub fn close(self) {
        // AsyncDevice Drop 时自动关闭
        log_info!("[Online] TUN 接口已关闭: {}", self.info.name);
    }
}

/// PoC 测试：创建 TUN 接口并读一个包（验证 wintun.dll 加载与权限）
///
/// 返回接口信息与读包结果（超时 1 秒，读不到包也算成功）。
pub async fn poc_create_and_recv() -> io::Result<VirtualNetInfo> {
    let mut net = VirtualNet::create("10.244.99.1", 24, 1400, None).await?;

    let mut buf = [0u8; 65535];
    // 用 tokio::time::timeout 包装，1 秒内读不到包也返回成功（说明接口可用）
    match tokio::time::timeout(std::time::Duration::from_secs(1), net.recv_packet(&mut buf)).await {
        Ok(Ok(len)) => {
            log_info!(
                "[Online] PoC 收到 IP 包: len={}, 前 20 字节={:02x?}",
                len,
                &buf[..len.min(20)]
            );
        }
        Ok(Err(e)) => {
            log_warn!("[Online] PoC 读包失败: {}", e);
            return Err(e);
        }
        Err(_) => {
            log_info!("[Online] PoC 1 秒内无包（接口正常，只是无流量）");
        }
    }

    let info = net.info().clone();
    net.close();
    Ok(info)
}

#[cfg(test)]
#[path = "tun_tests.rs"]
mod tests;
