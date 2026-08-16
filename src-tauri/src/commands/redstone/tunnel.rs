//! hongshi（红石联机）内核子进程封装。
//!
//! 以 `hongshi -server <addr> -port <mc_port> -status-file <tunnel.ini>` 拉起内核，
//! tunnel.ini 由内核原子写 `[tunnel] status=open/closed` 等隧道状态。

use crate::log_debug;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};

/// hongshi 隧道子进程句柄
pub struct HongshiTunnel {
    pub child: tokio::process::Child,
    pub server: String,
    pub status_file: PathBuf,
}

/// 从 tunnel.ini 解析出的隧道状态
pub struct TunnelStatus {
    pub status: String,
    pub server: Option<String>,
    pub port: Option<i64>,
    pub created: Option<String>,
}

/// 大小写不敏感读取 INI 段键值（INIFile 惯例：段名/键名不区分大小写）
fn ini_get_ci(ini: &crate::storage::ini::IniFile, section: &str, key: &str) -> Option<String> {
    ini.sections()
        .into_iter()
        .find(|s| s.eq_ignore_ascii_case(section))
        .and_then(|s| {
            ini.get_section(&s)
                .into_iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v)
        })
}

impl HongshiTunnel {
    /// 启动 hongshi 内核建立隧道（内部确保内核已释放就位；status-file 由内核原子写）
    ///
    /// 工作目录固定为内核所在目录（`<temp>/MoLaunch/hongshi/`），
    /// 内核按官方约定在 cwd 下写 `logs/<YYYY-MM-DD>.log` 日志。
    pub async fn spawn(server: &str, mc_port: u16, status_file: &Path) -> Result<Self, String> {
        let kernel_path = crate::resources::extract_hongshi_core()
            .map_err(|e| format!("释放红石内核失败: {e}"))?;
        if !kernel_path.is_file() {
            return Err(format!("红石内核不存在: {}", kernel_path.display()));
        }
        let kernel_dir = kernel_path
            .parent()
            .ok_or_else(|| "红石内核路径无效".to_string())?;
        let mut cmd = tokio::process::Command::new(&kernel_path);
        cmd.arg("-server")
            .arg(server)
            .arg("-port")
            .arg(mc_port.to_string())
            .arg("-status-file")
            .arg(status_file)
            .current_dir(kernel_dir)
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        #[cfg(windows)]
        {
            cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
        let mut child = cmd.spawn().map_err(|e| format!("启动红石内核失败: {e}"))?;

        if let Some(out) = child.stdout.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(out).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    log_debug!("[hongshi] {line}");
                }
            });
        }
        if let Some(err) = child.stderr.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    log_debug!("[hongshi] {line}");
                }
            });
        }

        Ok(Self {
            child,
            server: server.to_string(),
            status_file: status_file.to_path_buf(),
        })
    }

    /// 子进程是否仍在运行
    pub fn is_running(&mut self) -> bool {
        !matches!(self.child.try_wait(), Ok(Some(_)))
    }

    /// 停止子进程
    pub async fn stop(mut self) {
        let _ = self.child.kill().await;
    }
}

/// 解析 tunnel.ini 的 [tunnel] 段状态。
///
/// status 取 open/closed（大小写不敏感），否则 unknown；port 仅非 closed 状态
/// 下解析，缺失或 -1 视为未知；server/created 缺失时返回 None。
pub fn parse_tunnel_status(content: &str) -> TunnelStatus {
    let ini = crate::storage::ini::IniFile::parse(content);
    let status = match ini_get_ci(&ini, "tunnel", "status")
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("open") => "open".to_string(),
        Some("closed") => "closed".to_string(),
        _ => "unknown".to_string(),
    };
    let server = ini_get_ci(&ini, "tunnel", "server");
    let created = ini_get_ci(&ini, "tunnel", "created");
    let port = if status == "closed" {
        None
    } else {
        ini_get_ci(&ini, "tunnel", "port")
            .and_then(|s| s.parse::<i64>().ok())
            .filter(|&p| p != -1)
    };
    TunnelStatus {
        status,
        server,
        port,
        created,
    }
}

#[cfg(test)]
#[path = "tunnel_test.rs"]
mod tests;
