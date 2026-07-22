//! 工具模块（统一 IPC 入口）
//!
//! 对外只暴露 `tools_manager` 一个 IPC 命令，通过请求体的 `action` 字段分发到不同子模块。
//! 子模块：download（外部下载）/ filename（文件名获取）/ cleanup（清理垃圾）/ memory（内存优化）

pub mod cleanup;
pub mod download;
pub mod filename;
pub mod memory;
pub mod types;

use crate::state::AppState;
use tauri::State;
use types::*;

/// 统一工具 IPC 入口
#[tauri::command]
pub async fn tools_manager(
    state: State<'_, AppState>,
    req: ToolsRequest,
) -> Result<serde_json::Value, String> {
    match req.action.as_str() {
        // 外部下载
        "download_file" => {
            let p: DownloadFileParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            download::download_file(&state, p).await
        }
        "get_download_dir" => download::get_download_dir(&state).await,
        "list_downloads" => download::list_downloads(&state).await,
        "delete_download" => {
            let p: DeleteDownloadParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            download::delete_download(&state, p).await
        }
        // 文件名获取
        "fetch_filename" => {
            let p: FetchFilenameParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            filename::fetch_filename(p).await
        }
        // 清理游戏垃圾
        "cleanup_scan" => cleanup::scan(&state).await,
        "cleanup_execute" => {
            let p: CleanupExecuteParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            cleanup::execute(p).await
        }
        // 内存优化
        "memory_optimize" => {
            let p: MemoryOptimizeParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            memory::optimize(p).await
        }
        _ => Err(format!("未知操作: {}", req.action)),
    }
}
