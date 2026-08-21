//! easytier-core 子进程封装。
//!
//! 房主使用固定虚拟 IP（`-i 10.144.144.1`，与 Terracotta 标准一致），房客使用 `--dhcp` 动态分配；
//! 均通过 `--rpc-portal 127.0.0.1:动态端口` 暴露本地控制端口。房客可经随包附带的
//! easytier-cli 查询虚拟网络节点，自动发现房主联机中心（`discover_center`）。
//!
//! 默认 no-tun 模式（`--no-tun` 不创建虚拟网卡，走用户态转发）：进出虚拟网络的流量
//! 由 `add_port_forward` / `remove_port_forward`（easytier-cli port-forward）承担。

use crate::log_debug;
use crate::minecraft::online::scaffolding::server::CENTER_HOSTNAME_PREFIX;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

/// 房主默认虚拟 IP（Scaffolding 联机中心固定地址，与 Terracotta 标准 `10.144.144.1` 对齐）
pub const HOST_VIRTUAL_IP: &str = "10.144.144.1";

/// no-tun 用户态转发规则（对应 easytier-cli port-forward 的一条规则）
#[derive(Debug, Clone)]
pub struct PortForwardRule {
    pub proto: String,
    pub bind_addr: String,
    pub dst_addr: String,
}

/// 虚拟网络节点摘要（过滤中继：easytier 中继节点无虚拟 IP，不计入）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
    /// 节点设备码（easytier `peer list` 的 `id` 字段，全局唯一）
    pub node_id: String,
    /// 节点 hostname（房主为 `scaffolding-mc-server-{center_port}`，房客为自定义/默认）
    pub hostname: String,
    /// 虚拟 IP（如 `10.144.144.1`）
    pub virtual_ip: String,
    /// 是否本机节点（easytier `cost=Local` 表示本机）
    pub is_self: bool,
    /// 与中继节点的 RTT 延迟（ms，未测得时为 `-`）
    pub latency_ms: String,
}

/// easytier-core 子进程句柄
pub struct EasyTier {
    child: Child,
    rpc_portal: String,
    version: String,
    network_name: String,
    virtual_ip: Option<String>,
    cli_path: PathBuf,
}

/// 查询 easytier-core 版本（`--version` 输出形如 `easytier-core 2.6.4`，取第二段）
async fn query_version(core_path: &Path) -> String {
    match tokio::process::Command::new(core_path)
        .arg("--version")
        .output()
        .await
    {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    }
}

/// 申请一个空闲的本地端口（用于 rpc-portal 或本地转发）
pub(crate) async fn pick_free_port() -> Result<u16, String> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|e| format!("分配本地端口失败: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("读取本地端口失败: {e}"))?
        .port();
    drop(listener);
    Ok(port)
}

/// 组拼 easytier-core join 参数（不启动进程，便于单测）。
///
/// no-tun 模式下追加 `--no-tun`（用户态转发，不创建虚拟网卡）；
/// `extra` 原样透传（房主侧含 `--tcp-whitelist` / `--udp-whitelist` 白名单）。
pub(crate) fn build_join_args(
    network_name: &str,
    network_secret: &str,
    hostname: &str,
    rpc_portal: &str,
    ip: Option<&str>,
    no_tun: bool,
    extra: &[String],
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "--network-name".into(),
        network_name.to_string(),
        "--network-secret".into(),
        network_secret.to_string(),
        "--hostname".into(),
        hostname.to_string(),
        "-r".into(),
        rpc_portal.to_string(),
    ];
    match ip {
        Some(ip) => {
            args.push("-i".into());
            args.push(ip.to_string());
        }
        None => args.push("--dhcp".into()),
    }
    args.extend(extra.iter().cloned());
    if no_tun {
        args.push("--no-tun".into());
    }
    args
}

impl EasyTier {
    /// 启动 easytier-core 加入网络。
    ///
    /// `ip` 为 Some 时房主模式（`-i` 固定虚拟 IP），为 None 时房客模式（`--dhcp`）。
    /// `extra` 追加额外 CLI 参数（如 `--peers` 指定公共服务器、房主端口白名单）。
    /// `no_tun` 为 true 时不创建虚拟网卡（用户态转发），进出虚拟网络流量经
    /// `add_port_forward` / `remove_port_forward` 转发，无需管理员权限。
    #[allow(clippy::too_many_arguments)]
    pub async fn join(
        core_path: &Path,
        cli_path: &Path,
        network_name: &str,
        network_secret: &str,
        ip: Option<&str>,
        hostname: &str,
        extra: Vec<String>,
        no_tun: bool,
    ) -> Result<Self, String> {
        if !core_path.is_file() {
            return Err(format!(
                "easytier-core 不存在: {}（请在联机设置中指定正确路径）",
                core_path.display()
            ));
        }
        let rpc_port = pick_free_port().await?;
        let rpc_portal = format!("127.0.0.1:{rpc_port}");
        let args = build_join_args(
            network_name,
            network_secret,
            hostname,
            &rpc_portal,
            ip,
            no_tun,
            &extra,
        );

        let mut cmd = Command::new(core_path);
        cmd.args(&args)
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        #[cfg(windows)]
        {
            cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("启动 easytier-core 失败: {e}"))?;

        if let Some(out) = child.stdout.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(out).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    log_debug!("[easytier] {line}");
                }
            });
        }
        if let Some(err) = child.stderr.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    log_debug!("[easytier] {line}");
                }
            });
        }

        let version = query_version(core_path).await;

        Ok(Self {
            child,
            rpc_portal,
            version,
            network_name: network_name.to_string(),
            virtual_ip: ip.map(|s| s.to_string()),
            cli_path: cli_path.to_path_buf(),
        })
    }

    /// 当前 rpc-portal 地址
    pub fn rpc_portal(&self) -> &str {
        &self.rpc_portal
    }

    /// 虚拟网络名
    pub fn network_name(&self) -> &str {
        &self.network_name
    }

    /// 经 easytier-cli 从虚拟网络发现房主联机中心。
    ///
    /// 查询各节点 hostname，取 `scaffolding-mc-server-{center_port}` 前缀匹配的节点，
    /// 返回 (center_ip, center_port)。房客已在网络中时本机与房主同网段，可直接连接。
    pub async fn discover_center(&self) -> Result<(String, u16), String> {
        let nodes = self.easytier_cli(&["peer", "list"]).await?;
        let Some(nodes) = nodes.as_array() else {
            return Err("easytier-cli 输出不是 JSON 数组".to_string());
        };
        for node in nodes {
            let hostname = node.get("hostname").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(port_str) = hostname.strip_prefix(CENTER_HOSTNAME_PREFIX) {
                let port = port_str
                    .parse::<u16>()
                    .map_err(|e| format!("解析联机中心端口失败（{port_str}）: {e}"))?;
                let ip = node
                    .get("ipv4")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| format!("节点 {hostname} 缺少 ipv4 地址"))?;
                return Ok((ip.to_string(), port));
            }
        }
        Err(format!(
            "虚拟网络中未找到联机中心（{CENTER_HOSTNAME_PREFIX}*）"
        ))
    }

    /// 虚拟网络在线设备列表（`peer list` 过滤中继节点后，含本机）。
    ///
    /// easytier 中继节点（如 `PublicServer_moteam-servers`）无虚拟 IP（`ipv4` 为空），
    /// 据此过滤，保证组网人数/设备列表只反映真实在线设备。
    pub async fn peers(&self) -> Result<Vec<PeerInfo>, String> {
        let nodes = self.easytier_cli(&["peer", "list"]).await?;
        let Some(nodes) = nodes.as_array() else {
            return Err("easytier-cli 输出不是 JSON 数组".to_string());
        };
        let mut peers: Vec<PeerInfo> = Vec::new();
        for node in nodes {
            let ip = node.get("ipv4").and_then(|v| v.as_str()).unwrap_or("");
            if ip.is_empty() {
                continue; // 中继节点无虚拟 IP，过滤
            }
            peers.push(PeerInfo {
                node_id: node
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                hostname: node
                    .get("hostname")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                virtual_ip: ip.to_string(),
                is_self: node.get("cost").and_then(|v| v.as_str()) == Some("Local"),
                latency_ms: node
                    .get("lat_ms")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-")
                    .to_string(),
            });
        }
        Ok(peers)
    }

    /// 虚拟网络在线设备数（过滤中继节点后，含本机）。
    pub async fn peer_count(&self) -> Result<usize, String> {
        Ok(self.peers().await?.len())
    }

    /// 添加用户态端口转发（本地 `bind_addr` 监听到虚拟网络内 `dst_addr`）。
    ///
    /// no-tun 下无虚拟网卡，本地应用须经转发才能访问虚拟网络（房主侧白名单端口）。
    /// `port-forward add` 输出为文本，走 `easytier_cli_raw`（仅校验 exit code）。
    pub async fn add_port_forward(
        &self,
        proto: &str,
        bind_addr: &str,
        dst_addr: &str,
    ) -> Result<(), String> {
        self.easytier_cli_raw(&["port-forward", "add", proto, bind_addr, dst_addr])
            .await?;
        Ok(())
    }

    /// 移除用户态端口转发（按 `proto` + `bind_addr` 定位；规则不存在时同样返回失败）
    pub async fn remove_port_forward(&self, proto: &str, bind_addr: &str) -> Result<(), String> {
        self.easytier_cli_raw(&["port-forward", "remove", proto, bind_addr])
            .await?;
        Ok(())
    }

    /// 执行 easytier-cli 命令（`-p {rpc} -o json {args}`），成功时返回 stdout 原文。
    ///
    /// 注意：`port-forward add/remove` 等命令输出为文本而非 JSON（实测
    /// `Port forward rule add: ...`），必须经此入口执行，再按需解析。
    async fn easytier_cli_raw(&self, args: &[&str]) -> Result<Vec<u8>, String> {
        if !self.cli_path.is_file() {
            return Err(format!(
                "easytier-cli 不存在: {}（请与 easytier-core 同目录放置）",
                self.cli_path.display()
            ));
        }
        let mut cmd = Command::new(&self.cli_path);
        cmd.args(["-p", self.rpc_portal.as_str(), "-o", "json"])
            .args(args);
        #[cfg(windows)]
        {
            cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("执行 easytier-cli 失败: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "easytier-cli 执行失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(output.stdout)
    }

    /// 执行 easytier-cli 命令并解析 JSON 输出（`peer list` / `node info` 等）
    async fn easytier_cli(&self, args: &[&str]) -> Result<serde_json::Value, String> {
        let out = self.easytier_cli_raw(args).await?;
        serde_json::from_slice(&out).map_err(|e| format!("easytier-cli 输出解析失败: {e}"))
    }

    /// 本机虚拟 IP（房主固定 `10.144.144.1`；房客 DHCP 动态分配，未回显时为 None）
    pub fn virtual_ip(&self) -> Option<&str> {
        self.virtual_ip.as_deref()
    }

    /// 查询本机实际虚拟 IP。
    ///
    /// 房主（静态 `-i`）直接返回配置值；房客（`--dhcp`）经 easytier-cli `node info`
    /// 查询 DHCP 分配地址（`ipv4_addr` 形如 `10.144.144.2/24`，取前缀部分）。
    pub async fn query_virtual_ip(&self) -> Option<String> {
        if let Some(ip) = self.virtual_ip() {
            return Some(ip.to_string());
        }
        let Ok(info) = self.easytier_cli(&["node", "info"]).await else {
            return None;
        };
        info.get("ipv4_addr")
            .and_then(|v| v.as_str())
            .and_then(|s| s.split('/').next())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    /// easytier-core 版本号（`--version` 查询失败时为空字符串）
    pub fn version(&self) -> &str {
        &self.version
    }

    /// 子进程 PID（未运行则为 None）
    pub fn pid(&self) -> Option<u32> {
        self.child.id()
    }

    /// 停止子进程并等待退出
    pub async fn stop(mut self) {
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
    }
}
