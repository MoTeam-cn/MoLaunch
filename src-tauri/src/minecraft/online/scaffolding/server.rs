//! Scaffolding 联机中心：在虚拟 IP 上开放 TCP 服务，实现 §2.3 帧格式与五个标准协议。
//!
//! 帧格式：请求 `[类型长度u8][类型ascii][体长u32大端][体]`；
//! 响应 `[状态u8][体长u32大端][体]`，状态 0 成功 / 32-63 协议错 / 255 未知。

use crate::{log_debug, log_info, log_warn};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

/// 联机中心默认监听端口（被占时退回随机端口）
pub const DEFAULT_CENTER_PORT: u16 = 13448;

/// 中心 hostname 前缀：`scaffolding-mc-server-{center_port}`（标准语义，房客据此后缀发现联机中心端口）
pub const CENTER_HOSTNAME_PREFIX: &str = "scaffolding-mc-server-";

/// 标准协议列表（`\0` 分隔，用于 c:protocols 响应）
const PROTOCOLS: &[u8] =
    b"c:ping\0c:protocols\0c:server_port\0c:player_ping\0c:player_profiles_list";

/// 玩家条目
#[derive(Debug, Clone)]
struct PlayerEntry {
    name: String,
    machine_id: String,
    vendor: String,
    kind: String,
}

/// 联机中心共享状态
#[derive(Clone)]
pub struct ScaffoldingServerState {
    mc_port: Arc<Mutex<Option<u16>>>,
    players: Arc<Mutex<Vec<PlayerEntry>>>,
}

impl Default for ScaffoldingServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ScaffoldingServerState {
    /// 创建空的联机中心状态
    pub fn new() -> Self {
        Self {
            mc_port: Arc::new(Mutex::new(None)),
            players: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 由房主侧更新 MC 服务器端口（None 表示未启动）
    pub fn set_mc_port(&self, port: Option<u16>) {
        *self.mc_port.lock().unwrap() = port;
    }

    /// 设置房主档案（kind = HOST）
    pub fn set_host_profile(&self, name: String, machine_id: String, vendor: String) {
        let mut players = self.players.lock().unwrap();
        players.retain(|p| p.kind != "HOST");
        players.push(PlayerEntry {
            name,
            machine_id,
            vendor,
            kind: "HOST".into(),
        });
    }
}

/// Scaffolding 联机中心 TCP 服务
pub struct ScaffoldingServer {
    port: u16,
    state: ScaffoldingServerState,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl ScaffoldingServer {
    /// 启动联机中心，默认监听 `0.0.0.0:13448`（含虚拟 IP），被占则退为随机端口
    pub async fn start() -> Result<Self, String> {
        Self::start_on(ScaffoldingServerState::new()).await
    }

    /// 启动联机中心并挂载外部共享状态（MC 端口由房主探测任务写入）
    pub async fn start_on(state: ScaffoldingServerState) -> Result<Self, String> {
        let (listener, port) = match TcpListener::bind(("0.0.0.0", DEFAULT_CENTER_PORT)).await {
            Ok(listener) => {
                let port = listener
                    .local_addr()
                    .map_err(|e| format!("读取联机中心端口失败: {e}"))?
                    .port();
                (listener, port)
            }
            Err(_) => {
                let listener = TcpListener::bind(("0.0.0.0", 0))
                    .await
                    .map_err(|e| format!("启动联机中心失败: {e}"))?;
                let port = listener
                    .local_addr()
                    .map_err(|e| format!("读取联机中心端口失败: {e}"))?
                    .port();
                log_warn!("联机中心端口 {DEFAULT_CENTER_PORT} 被占用，改用随机端口 {port}");
                (listener, port)
            }
        };

        let stop = Arc::new(AtomicBool::new(false));
        let accept_stop = stop.clone();
        let accept_state = state.clone();
        let handle = tokio::spawn(async move {
            loop {
                let (stream, peer) = match listener.accept().await {
                    Ok(x) => x,
                    Err(_) => break, // listener 关闭
                };
                let state = accept_state.clone();
                tokio::spawn(async move {
                    log_debug!("[scaffolding] 客户端接入: {peer}");
                    let _ = handle_client(stream, state).await;
                });
            }
            let _ = accept_stop;
        });

        log_info!("[scaffolding] 联机中心已启动，端口 {port}");
        Ok(Self {
            port,
            state,
            stop,
            handle: Some(handle),
        })
    }

    /// 实际监听端口（可能因占用而不同于默认值）
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 中心 hostname：`scaffolding-mc-server-{center_port}`（真实联机中心端口，含端口被占退为随机端口的情况）
    pub fn hostname(&self) -> String {
        format!("{CENTER_HOSTNAME_PREFIX}{}", self.port)
    }

    /// 共享状态（供房主更新 MC 端口 / 房主档案）
    pub fn state(&self) -> &ScaffoldingServerState {
        &self.state
    }

    /// 停止服务（abort accept 循环以释放监听端口）
    pub async fn stop(self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle {
            handle.abort();
            let _ = handle.await;
        }
    }
}

/// 处理单条连接：循环读请求 → 处理 → 写响应
async fn handle_client(mut stream: TcpStream, state: ScaffoldingServerState) -> Result<(), ()> {
    loop {
        let Some((kind, body)) = read_request(&mut stream).await? else {
            return Ok(());
        };
        let (status, data) = handle_request(&kind, &body, &state);
        write_response(&mut stream, status, &data).await?;
    }
}

/// 读取请求帧，连接关闭返回 Ok(None)，协议错误返回 Err
async fn read_request(stream: &mut TcpStream) -> Result<Option<(String, Vec<u8>)>, ()> {
    let mut len_buf = [0u8; 1];
    let n = stream.read(&mut len_buf).await.map_err(|_| ())?;
    if n == 0 {
        return Ok(None);
    }
    let kind_len = len_buf[0] as usize;
    if kind_len == 0 || kind_len > 128 {
        return Err(());
    }
    let mut kind = vec![0u8; kind_len];
    stream.read_exact(&mut kind).await.map_err(|_| ())?;
    let mut body_len_buf = [0u8; 4];
    stream.read_exact(&mut body_len_buf).await.map_err(|_| ())?;
    let body_len = u32::from_be_bytes(body_len_buf) as usize;
    if body_len > 1024 * 1024 {
        return Err(());
    }
    let mut body = vec![0u8; body_len];
    if body_len > 0 {
        stream.read_exact(&mut body).await.map_err(|_| ())?;
    }
    Ok(Some((String::from_utf8_lossy(&kind).to_string(), body)))
}

/// 写出响应帧
async fn write_response(stream: &mut TcpStream, status: u8, data: &[u8]) -> Result<(), ()> {
    stream.write_all(&[status]).await.map_err(|_| ())?;
    stream
        .write_all(&(data.len() as u32).to_be_bytes())
        .await
        .map_err(|_| ())?;
    stream.write_all(data).await.map_err(|_| ())?;
    stream.flush().await.map_err(|_| ())
}

/// 处理请求，返回 (状态, 响应体)
fn handle_request(kind: &str, body: &[u8], state: &ScaffoldingServerState) -> (u8, Vec<u8>) {
    let Some((ns, cmd)) = kind.split_once(':') else {
        return (255, b"invalid request type".to_vec());
    };
    if ns != "c" {
        return (255, format!("unknown namespace: {ns}").into_bytes());
    }
    match cmd {
        "ping" => (0, body.to_vec()),
        "protocols" => {
            let supported: Vec<&str> = std::str::from_utf8(PROTOCOLS)
                .map(|s| s.split('\0').filter(|p| !p.is_empty()).collect())
                .unwrap_or_default();
            let requested: Vec<&str> = std::str::from_utf8(body)
                .map(|s| s.split('\0').filter(|p| !p.is_empty()).collect())
                .unwrap_or_default();
            let intersection: Vec<&str> = supported
                .iter()
                .filter(|p| requested.is_empty() || requested.contains(p))
                .copied()
                .collect();
            (0, intersection.join("\0").into_bytes())
        }
        "server_port" => match *state.mc_port.lock().unwrap() {
            Some(port) => (0, port.to_be_bytes().to_vec()),
            None => (32, Vec::new()),
        },
        "player_ping" => handle_player_ping(body, state),
        "player_profiles_list" => handle_player_profiles_list(state),
        _ => (255, format!("unknown protocol: {cmd}").into_bytes()),
    }
}

/// c:player_ping：JSON 注册 / 心跳，返回状态 0
fn handle_player_ping(body: &[u8], state: &ScaffoldingServerState) -> (u8, Vec<u8>) {
    let Ok(value) = serde_json::from_slice::<Value>(body) else {
        return (1, b"invalid json".to_vec());
    };
    let Some(machine_id) = value.get("machine_id").and_then(|v| v.as_str()) else {
        return (1, b"missing machine_id".to_vec());
    };
    let name = value.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let vendor = value.get("vendor").and_then(|v| v.as_str()).unwrap_or("");
    let mut players = state.players.lock().unwrap();
    if let Some(entry) = players.iter_mut().find(|p| p.machine_id == machine_id) {
        if entry.kind != "HOST" {
            entry.name = name.to_string();
            entry.vendor = vendor.to_string();
        }
    } else {
        players.push(PlayerEntry {
            name: name.to_string(),
            machine_id: machine_id.to_string(),
            vendor: vendor.to_string(),
            kind: "GUEST".into(),
        });
    }
    (0, Vec::new())
}

/// c:player_profiles_list：返回玩家列表 JSON
fn handle_player_profiles_list(state: &ScaffoldingServerState) -> (u8, Vec<u8>) {
    let players = state.players.lock().unwrap();
    let list: Vec<Value> = players
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "machine_id": p.machine_id,
                "vendor": p.vendor,
                "kind": p.kind,
            })
        })
        .collect();
    (0, serde_json::to_vec(&list).unwrap_or_default())
}
