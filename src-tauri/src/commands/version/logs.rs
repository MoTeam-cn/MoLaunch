//! 实例日志读取命令（版本设置页「实例日志」Tab）
//!
//! 两个 action：`list_instance_logs` 列出实例 `logs/` 目录下的日志文件；
//! `read_instance_log` 读取指定日志内容（.log 直接读，.log.gz 先解压）。
//! 读取前校验文件名（防路径穿越），返回前脱敏（复用 logger::sanitize）。

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DirParams {
    dir: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadLogParams {
    dir: String,
    name: String,
}

/// 实例日志文件条目（列表返回）
#[derive(Debug, Clone, Serialize)]
pub struct InstanceLogFile {
    pub name: String,
    pub size: u64,
    pub modified: u64,
}

/// 列出实例 logs 目录下的日志文件（.log / .log.gz，按修改时间倒序）
fn list_instance_logs_inner(dir: &str) -> Vec<InstanceLogFile> {
    let logs_dir = std::path::Path::new(dir).join("logs");
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&logs_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".log") && !name.ends_with(".log.gz") {
                continue;
            }
            let md = entry.metadata().ok();
            files.push(InstanceLogFile {
                size: md.as_ref().map(|m| m.len()).unwrap_or(0),
                modified: md
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                name,
            });
        }
    }
    files.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| b.name.cmp(&a.name))
    });
    files
}

/// 读取日志文件内容（非 UTF-8 字节用 lossy 替换，避免整文件读取失败）
fn read_log_bytes(path: &std::path::Path) -> Result<String, std::io::Error> {
    let bytes = if path.as_os_str().to_string_lossy().ends_with(".gz") {
        use std::io::Read;
        let file = std::fs::File::open(path)?;
        let mut gz = flate2::read::GzDecoder::new(file);
        let mut buf = Vec::new();
        gz.read_to_end(&mut buf)?;
        buf
    } else {
        std::fs::read(path)?
    };
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// 读取实例日志内容：校验文件名（防路径穿越）→ 读取/解压 → 脱敏
fn read_instance_log_inner(dir: &str, name: &str) -> Result<String, String> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || !crate::utils::path::is_safe_relative_path(name)
        || !(name.ends_with(".log") || name.ends_with(".log.gz"))
    {
        return Err(format!("非法日志文件名: {}", name));
    }
    let path = std::path::Path::new(dir).join("logs").join(name);
    let content = read_log_bytes(&path).map_err(|e| format!("读取日志文件失败: {}", e))?;
    Ok(crate::logger::sanitize_sensitive_info(&content))
}

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "list_instance_logs",
        handler!(_state, _app, params, {
            let p: DirParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            serde_json::to_value(list_instance_logs_inner(&p.dir)).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "read_instance_log",
        handler!(_state, _app, params, {
            let p: ReadLogParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            serde_json::to_value(read_instance_log_inner(&p.dir, &p.name)?)
                .map_err(|e| e.to_string())
        }),
    );

    d
});

/// 分发入口
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}
