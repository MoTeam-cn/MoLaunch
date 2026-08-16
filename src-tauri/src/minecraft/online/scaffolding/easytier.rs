//! easytier-core 子进程封装。
//!
//! 房主使用固定虚拟 IP（`-i 10.144.144.1`，与 Terracotta 标准一致），房客使用 `--dhcp` 动态分配；
//! 均通过 `--rpc-portal 127.0.0.1:动态端口` 暴露本地控制端口。房客可经随包附带的
//! easytier-cli 查询虚拟网络节点，自动发现房主联机中心（`discover_center`）。

use crate::log_debug;
use crate::minecraft::online::scaffolding::server::CENTER_HOSTNAME_PREFIX;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

/// 房主默认虚拟 IP（Scaffolding 联机中心固定地址，与 Terracotta 标准 `10.144.144.1` 对齐）
pub const HOST_VIRTUAL_IP: &str = "10.144.144.1";

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

/// 申请一个空闲的本地端口（用于 rpc-portal）
async fn pick_free_port() -> Result<u16, String> {
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

impl EasyTier {
    /// 启动 easytier-core 加入网络。
    ///
    /// `ip` 为 Some 时房主模式（`-i` 固定虚拟 IP），为 None 时房客模式（`--dhcp`）。
    /// `extra` 追加额外 CLI 参数（如 `--peers` 指定公共服务器）。
    pub async fn join(
        core_path: &Path,
        cli_path: &Path,
        network_name: &str,
        network_secret: &str,
        ip: Option<&str>,
        hostname: &str,
        extra: Vec<String>,
    ) -> Result<Self, String> {
        if !core_path.is_file() {
            return Err(format!(
                "easytier-core 不存在: {}（请在联机设置中指定正确路径）",
                core_path.display()
            ));
        }
        let rpc_port = pick_free_port().await?;
        let rpc_portal = format!("127.0.0.1:{rpc_port}");

        let mut args: Vec<String> = vec![
            "--network-name".into(),
            network_name.to_string(),
            "--network-secret".into(),
            network_secret.to_string(),
            "--hostname".into(),
            hostname.to_string(),
            "-r".into(),
            rpc_portal.clone(),
        ];
        match ip {
            Some(ip) => {
                args.push("-i".into());
                args.push(ip.to_string());
            }
            None => args.push("--dhcp".into()),
        }
        args.extend(extra);

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
        if !self.cli_path.is_file() {
            return Err(format!(
                "easytier-cli 不存在: {}（请与 easytier-core 同目录放置）",
                self.cli_path.display()
            ));
        }
        let mut cmd = Command::new(&self.cli_path);
        cmd.args(["-p", &self.rpc_portal, "-o", "json", "peer", "list"]);
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
                "easytier-cli 查询节点失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let nodes: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("easytier-cli 输出解析失败: {e}"))?;
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

    /// 本机虚拟 IP（房主固定 `10.144.144.1`；房客 DHCP 动态分配，未回显时为 None）
    pub fn virtual_ip(&self) -> Option<&str> {
        self.virtual_ip.as_deref()
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
