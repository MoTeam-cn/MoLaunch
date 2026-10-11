//! hongshic（红石联机）内核子进程封装。
//!
//! 以 `hongshic -t <relay> -p <game_port>` 拉起内核。新版内核**没有配置文件**：
//! 隧道地址从 stdout 的 `endpoint=` 字段读取（形如 `endpoint=nj.hongshi.site:41269`），
//! 旧版的 `-status-file` / `tunnel.ini` 机制已随 Redstone Online2 一并移除。

use crate::log_debug;
use crate::log_info;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};

/// hongshic 隧道子进程句柄
pub struct HongshiTunnel {
    pub child: tokio::process::Child,
    /// 中继地址（玩家侧看到的服务器）
    pub relay: String,
    /// 本地 MC 端口
    pub mc_port: u16,
    /// 内核 stdout 解析出的 endpoint（`host:port`），隧道建立后才有值
    endpoint: Arc<Mutex<Option<String>>>,
    /// 内核可执行文件路径（日志与排障用）
    pub kernel_path: PathBuf,
    /// 隧道启动时间（本地时间字符串，供前端展示）
    pub created_at: String,
}

impl HongshiTunnel {
    /// 启动内核建立隧道
    ///
    /// 内核来源由 [`crate::commands::redstone::kernel::ensure_kernel`] 决定
    /// （AppData 缓存 → 运行时下载 → 编译期兜底）。
    /// 工作目录固定为内核日志目录，内核按官方约定在 cwd 下写 `logs/<YYYY-MM-DD>.log`。
    pub async fn spawn(relay: &str, mc_port: u16) -> Result<Self, String> {
        let (kernel_path, source) = crate::commands::redstone::kernel::ensure_kernel().await?;
        if !kernel_path.is_file() {
            return Err(format!("红石内核不存在: {}", kernel_path.display()));
        }

        let work_dir = crate::resources::hongshi_kernel_dir();
        std::fs::create_dir_all(&work_dir).map_err(|e| format!("创建内核工作目录失败: {e}"))?;

        let mut cmd = tokio::process::Command::new(&kernel_path);
        cmd.arg("-t")
            .arg(relay)
            .arg("-p")
            .arg(mc_port.to_string())
            .current_dir(&work_dir)
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        #[cfg(windows)]
        {
            cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
        let mut child = cmd.spawn().map_err(|e| format!("启动红石内核失败: {e}"))?;

        // stdout：边落日志边抓 endpoint（官方约定 endpoint= 之后到下一个空白即玩家地址）
        let endpoint: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        if let Some(out) = child.stdout.take() {
            let slot = Arc::clone(&endpoint);
            tokio::spawn(async move {
                let mut reader = BufReader::new(out).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    if let Some(found) = parse_endpoint(&line) {
                        if let Ok(mut guard) = slot.lock() {
                            *guard = Some(found);
                        }
                    }
                    log_debug!("[hongshic] {line}");
                }
            });
        }
        if let Some(err) = child.stderr.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    log_debug!("[hongshic] {line}");
                }
            });
        }

        log_info!(
            "[Redstone] 内核已拉起: source={}, relay={}, mc_port={}, kernel={}",
            source.as_str(),
            relay,
            mc_port,
            kernel_path.display()
        );

        Ok(Self {
            child,
            relay: relay.to_string(),
            mc_port,
            endpoint,
            kernel_path,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    /// 子进程是否仍在运行
    pub fn is_running(&mut self) -> bool {
        !matches!(self.child.try_wait(), Ok(Some(_)))
    }

    /// 读取已建立的隧道地址（未建立时为 None）
    pub fn endpoint(&self) -> Option<String> {
        self.endpoint.lock().ok().and_then(|g| g.clone())
    }

    /// 停止子进程
    pub async fn stop(mut self) {
        let _ = self.child.kill().await;
    }
}

/// 从内核一行输出中提取 endpoint 地址
///
/// 官方约定：`endpoint=` 之前的内容都是装饰、可能变化；之后取到下一个空白为止，
/// 即玩家要填进「直接连接」的地址（如 `nj.hongshi.site:41269`）。
pub fn parse_endpoint(line: &str) -> Option<String> {
    const MARKER: &str = "endpoint=";
    let rest = &line[line.find(MARKER)? + MARKER.len()..];
    // 值必须紧跟等号：紧跟空白说明没有取值，不能顺延到下一个 token
    match rest.chars().next() {
        Some(c) if !c.is_whitespace() => {}
        _ => return None,
    }
    let addr = rest.split_whitespace().next().unwrap_or("");
    if addr.is_empty() {
        None
    } else {
        Some(addr.to_string())
    }
}

/// 把 `host:port` 拆成（host, port）
///
/// 端口缺失或非法时返回（原串, None）；无冒号时整体视为 host。
pub fn split_endpoint(endpoint: &str) -> (String, Option<i64>) {
    match endpoint.rsplit_once(':') {
        Some((host, port)) => match port.parse::<i64>() {
            Ok(p) => (host.to_string(), Some(p)),
            Err(_) => (endpoint.to_string(), None),
        },
        None => (endpoint.to_string(), None),
    }
}

#[cfg(test)]
#[path = "tunnel_test.rs"]
mod tests;
