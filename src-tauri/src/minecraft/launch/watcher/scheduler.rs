//! 事件分发与日志监控调度
//!
//! 负责 stdout/stderr 读取、加载进度检测、联机端口事件与退出/崩溃分析调度。

use super::log_parser::detect_load_progress;
use super::log_reader::{read_logs, tail_latest_log};
use super::process::GameWatcher;
use super::types::{ExitInfo, GameState};
use crate::log_info;
use regex::Regex;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, OnceLock};
use tauri::Emitter;
use tokio::sync::Mutex;

/// 联机模块 MC 局域网端口检测事件名
///
/// GameWatcher 从游戏进程日志（stdout/latest.log）或进程监听端口检测到
/// MC 局域网端口时，通过此事件通知前端联机模块。payload 为 u16 端口号。
pub const ONLINE_MC_PORT_DETECTED_EVENT: &str = "online://mc-port-detected";

/// MC 开放局域网时日志中的端口正则
///
/// 覆盖各版本实际输出格式：
/// - "Started on 4053"（1.8-1.12，1.12.2 实测确认）
/// - "Local game hosted on port 49152"（更早版本）
/// - "Published server on 192.168.1.100:49152" / "Started serving on ..."（1.13+）
static LAN_PORT_RE: OnceLock<Regex> = OnceLock::new();

fn lan_port_regex() -> &'static Regex {
    LAN_PORT_RE.get_or_init(|| {
        Regex::new(
            r"(?:Started on|Local game hosted on port) (\d{1,5})|(?:Published server|Started serving) on .*:(\d{1,5})",
        )
        .expect("LAN port regex 编译失败")
    })
}

/// 从日志行解析 MC 局域网端口（无匹配返回 None）
fn parse_lan_port(line: &str) -> Option<u16> {
    let caps = lan_port_regex().captures(line)?;
    caps.get(1)
        .or_else(|| caps.get(2))
        .and_then(|m| m.as_str().parse::<u16>().ok())
        .filter(|&p| p > 0)
}

/// 统一上报 MC 局域网端口（日志 / 监听端口轮询双信号共用）
///
/// last_port 记录最近已上报端口，双信号下避免重复 emit。
async fn report_lan_port(port: u16, app_handle: &Option<tauri::AppHandle>, last_port: &AtomicU16) {
    if port == 0 {
        return;
    }
    if last_port.swap(port, Ordering::Relaxed) == port {
        return;
    }
    log_info!(
        "[Watcher] 检测到 MC 局域网端口: {}（联机模块自动捕获）",
        port
    );
    if let Some(ref handle) = app_handle {
        let _ = handle.emit(ONLINE_MC_PORT_DETECTED_EVENT, port);
    }
}

/// stdout/latest.log 每行统一入口：匹配 MC 局域网端口则上报
async fn emit_lan_port_if_matched(
    line: &str,
    app_handle: &Option<tauri::AppHandle>,
    last_port: &AtomicU16,
) {
    if let Some(port) = parse_lan_port(line) {
        report_lan_port(port, app_handle, last_port).await;
    }
}

/// 枚举指定进程监听的 TCP 端口（排除回环地址）
///
/// 基于 netstat2 直接读取系统套接字表，不依赖游戏日志格式与 stdout 可用性；
/// MC 开放局域网后由 Java 进程监听一个非回环 TCP 端口，据此自动识别上报。
fn listening_tcp_ports(pid: u32) -> Vec<u16> {
    let af_flags = netstat2::AddressFamilyFlags::all();
    let proto_flags = netstat2::ProtocolFlags::TCP;
    let Ok(sockets) = netstat2::get_sockets_info(af_flags, proto_flags) else {
        return Vec::new();
    };
    let mut ports = Vec::new();
    for sock in sockets {
        if !sock.associated_pids.contains(&pid) {
            continue;
        }
        if let netstat2::ProtocolSocketInfo::Tcp(tcp) = sock.protocol_socket_info {
            if tcp.state != netstat2::TcpState::Listen {
                continue;
            }
            // 回环监听多为 JVM 内部服务（RMI 等），排除以降低误报
            if tcp.local_addr.is_loopback() {
                continue;
            }
            ports.push(tcp.local_port);
        }
    }
    ports.sort_unstable();
    ports.dedup();
    ports
}

impl GameWatcher {
    /// 开始监控
    pub async fn start_monitoring(
        &self,
        child: tokio::process::Child,
    ) -> Arc<Mutex<Option<tokio::process::Child>>> {
        let child_handle = Arc::new(Mutex::new(Some(child)));
        let child_clone = child_handle.clone();

        // 双来源端口检测共享状态：最近上报端口（去重）与进程退出标记（latest.log 兜底任务退出用）
        let last_port = Arc::new(AtomicU16::new(0));
        let process_exited = Arc::new(AtomicBool::new(false));

        // 获取stdout和stderr
        let (stdout, stderr) = {
            let mut guard = child_clone.lock().await;
            if let Some(ref mut c) = *guard {
                (c.stdout.take(), c.stderr.take())
            } else {
                (None, None)
            }
        };

        // 启动日志读取
        if let Some(stdout) = stdout {
            let log_buffer = self.log_buffer.clone();
            let state = self.state.clone();
            let load_progress = self.load_progress.clone();
            let max_lines = self.max_log_lines;
            let app_handle = self.app_handle.clone();
            let last_port_clone = last_port.clone();
            let exited = process_exited.clone();

            tokio::spawn(async move {
                read_logs(stdout, "stdout", log_buffer, max_lines, move |line| {
                    let state = state.clone();
                    let load_progress = load_progress.clone();
                    let app_handle = app_handle.clone();
                    let last_port = last_port_clone.clone();
                    async move {
                        let new_progress = detect_load_progress(&line);
                        {
                            let mut current = load_progress.write().await;
                            if new_progress > *current {
                                *current = new_progress;
                            }
                        }

                        {
                            let mut state_guard = state.write().await;
                            if *state_guard == GameState::Starting {
                                *state_guard = GameState::Loading;
                            }
                        }

                        emit_lan_port_if_matched(&line, &app_handle, &last_port).await;
                    }
                })
                .await;
                exited.store(true, Ordering::Relaxed);
            });
        }

        // 兜底：增量监控 logs/latest.log（MC File appender 保证写入），stdout 无日志时仍能捕获端口
        let log_path = self.game_dir.join("logs").join("latest.log");
        let exited = process_exited.clone();
        let app_handle = self.app_handle.clone();
        let last_port_clone = last_port.clone();
        tokio::spawn(async move {
            tail_latest_log(log_path, exited, move |line| {
                let app_handle = app_handle.clone();
                let last_port = last_port_clone.clone();
                async move {
                    emit_lan_port_if_matched(&line, &app_handle, &last_port).await;
                }
            })
            .await;
        });

        // 端口增强：轮询游戏进程监听端口（不依赖日志格式与 stdout）
        // MC 开放局域网即出现新的非回环监听端口，连续两次轮询确认后上报
        let pid = self.pid;
        let exited = process_exited.clone();
        let app_handle = self.app_handle.clone();
        let last_port_clone = last_port.clone();
        tokio::spawn(async move {
            let mut seen: std::collections::HashMap<u16, u32> = std::collections::HashMap::new();
            loop {
                if exited.load(Ordering::Relaxed) {
                    break;
                }
                let current = listening_tcp_ports(pid);
                for &port in &current {
                    let count = seen.entry(port).or_insert(0);
                    *count += 1;
                    if *count == 2 {
                        report_lan_port(port, &app_handle, &last_port_clone).await;
                    }
                }
                seen.retain(|port, _| current.contains(port));
                tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
            }
        });

        // 读取stderr
        if let Some(stderr) = stderr {
            let log_buffer = self.log_buffer.clone();
            let max_lines = self.max_log_lines;

            tokio::spawn(async move {
                read_logs(stderr, "stderr", log_buffer, max_lines, |_| async {}).await;
            });
        }

        // 启动窗口标题修改：轮询找到 MC 窗口并改写标题，支持 {date}/{time} 实时替换
        // 空标题（用户留空"跟随全局设置"）不触发改写，避免把游戏窗口标题改成空白
        if let Some(ref title) = self.window_title {
            if !title.trim().is_empty() {
                let title = title.clone();
                let pid = self.pid;
                tokio::spawn(async move {
                    super::window_title::apply_window_title(pid, title).await;
                });
            }
        }

        // 启动状态检测
        let state = self.state.clone();
        let _load_progress = self.load_progress.clone();
        let log_buffer = self.log_buffer.clone();
        let _pid = self.pid;
        let exit_tx = self.exit_tx.clone();
        let version_id = self.version_id.clone();
        let game_dir = self.game_dir.clone();
        let manual_stop = self.manual_stop.clone();
        let process_exited_clone = process_exited.clone();

        tokio::spawn(async move {
            // 等待进程结束
            let exit_code = {
                let mut guard = child_clone.lock().await;
                if let Some(ref mut c) = *guard {
                    c.wait().await.ok().map(|s| s.code().unwrap_or(-1))
                } else {
                    None
                }
            };
            process_exited_clone.store(true, Ordering::Relaxed);

            let exit_code = exit_code.unwrap_or(-1);
            let logs = {
                let buffer = log_buffer.lock().await;
                buffer.iter().cloned().collect::<Vec<_>>()
            };

            // 分析是否崩溃
            // 延迟 2 秒让文件系统刷新崩溃报告
            // 修复：手动停止（stop_game）时跳过崩溃分析，直接按正常退出处理
            let is_manual_stop = manual_stop.load(std::sync::atomic::Ordering::Relaxed);
            let crash_info = if exit_code != 0 && !is_manual_stop {
                log_info!(
                    "[Watcher] 游戏异常退出（code={}），2 秒后开始崩溃分析...",
                    exit_code
                );
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                super::analyzer::analyze_crash(exit_code, &logs, &game_dir).await
            } else if is_manual_stop {
                log_info!(
                    "[Watcher] 游戏被手动停止（code={}），跳过崩溃分析",
                    exit_code
                );
                None
            } else {
                None
            };

            let exit_info = if let Some(info) = crash_info {
                log_info!(
                    "[Watcher] 崩溃分析完成: {}（类别: {:?}）",
                    info.reason,
                    info.category
                );
                let mut state_guard = state.write().await;
                *state_guard = GameState::Crashed(info.clone());
                ExitInfo {
                    code: exit_code,
                    is_normal: false,
                    crash_info: Some(info),
                }
            } else {
                let exit_info = ExitInfo {
                    code: exit_code,
                    // 手动停止或退出码为 0 都算正常退出
                    is_normal: exit_code == 0 || is_manual_stop,
                    crash_info: None,
                };
                let mut state_guard = state.write().await;
                *state_guard = GameState::Exited(exit_info.clone());
                exit_info
            };

            // 发送退出通知
            let _ = exit_tx.send(Some(exit_info));

            log_info!(
                "[Watcher] Game process exited (PID: {}, code: {}, version: {})",
                _pid,
                exit_code,
                version_id
            );
        });

        child_handle
    }
}

#[cfg(test)]
mod tests {
    use super::parse_lan_port;

    #[test]
    fn parse_lan_port_matches_common_formats() {
        // 1.12.2 等旧版实测格式
        assert_eq!(
            parse_lan_port("[16:34:49] [Client thread/INFO]: Started on 4053"),
            Some(4053)
        );
        // 更早版本
        assert_eq!(
            parse_lan_port("[Server thread/INFO]: Local game hosted on port 49152"),
            Some(49152)
        );
        // 1.13+ 带 IP 的格式
        assert_eq!(
            parse_lan_port("[Server thread/INFO]: Published server on 192.168.1.100:49152"),
            Some(49152)
        );
        assert_eq!(
            parse_lan_port("[Server thread/INFO]: Started serving on 192.168.1.100:25565"),
            Some(25565)
        );
    }

    #[test]
    fn parse_lan_port_ignores_unrelated_lines() {
        assert_eq!(
            parse_lan_port(r#"[Server thread/INFO]: Preparing level "world""#),
            None
        );
        assert_eq!(
            parse_lan_port("[Client thread/INFO]: Started on world gen"),
            None
        );
        assert_eq!(parse_lan_port(""), None);
    }
}
