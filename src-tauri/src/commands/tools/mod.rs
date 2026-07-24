//! 工具模块（统一 IPC 入口）
//!
//! 对外只暴露 `tools_manager` 一个 IPC 命令，通过请求体的 `action` 字段分发到不同子模块。
//! 子模块：download（外部下载）/ filename（文件名获取）/ cleanup（清理垃圾）/ memory（内存优化）
//! / mod_tools（Mod 依赖检测 + 去重）/ data_export（启动器数据导出）/ crash_analyzer（崩溃日志分析）
//! / screenshot（截图管理）/ resourcepack（资源包管理）/ version_json（版本 JSON 读写）
//! / archive（存档管理）/ network（网络延迟 + 服务器状态）/ nbt（NBT 数据查看）
//!
//! 注：种子地图工具（seedmap）已迁移至前端 WASM 方案，不再走后端 IPC。
//! cubiomes C 库通过 Emscripten 编译为 WebAssembly，前端 Worker 直接调用，
//! 后端只通过 `res://` 协议提供 .wasm/.js 文件（见 res_scheme.rs + build.rs）。

pub mod archive;
pub mod cleanup;
pub mod crash_analyzer;
pub mod data_export;
pub mod download;
pub mod filename;
pub mod memory;
pub mod mod_tools;
pub mod nbt;
pub mod network;
pub mod resourcepack;
pub mod screenshot;
pub mod types;
pub mod version_json;

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
        // Mod 依赖检测
        "mod_dependency_check" => {
            let p: ModDependencyCheckParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            mod_tools::mod_dependency_check(&state, p).await
        }
        // Mod 去重扫描
        "mod_dedup_scan" => {
            let p: ModDedupScanParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            mod_tools::mod_dedup_scan(&state, p).await
        }
        // 启动器数据导出
        "export_launcher_data" => {
            let p: ExportLauncherDataParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            data_export::export_launcher_data(&state, p).await
        }
        // 崩溃日志分析
        "crash_analyze" => {
            let p: CrashAnalyzeParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            crash_analyzer::analyze(&state, p).await
        }
        // 截图管理
        "screenshot_list" => {
            let p: ScreenshotListParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            screenshot::list(&state, p).await
        }
        "screenshot_delete" => {
            let p: ScreenshotDeleteParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            screenshot::delete(&state, p).await
        }
        // 资源包管理
        "resourcepack_list" => {
            let p: ResourcePackListParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            resourcepack::list(&state, p).await
        }
        "resourcepack_convert" => {
            let p: ResourcePackConvertParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            resourcepack::convert(&state, p).await
        }
        // 版本 JSON 读写
        "version_json_read" => {
            let p: VersionJsonReadParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            version_json::read(&state, p).await
        }
        "version_json_save" => {
            let p: VersionJsonSaveParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            version_json::save(&state, p).await
        }
        // 存档管理
        "archive_list" => {
            let p: ArchiveListParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            archive::list(&state, p).await
        }
        "archive_backup" => {
            let p: ArchiveBackupParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            archive::backup(&state, p).await
        }
        "archive_restore" => {
            let p: ArchiveRestoreParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            archive::restore(&state, p).await
        }
        // 网络延迟测试
        "network_latency_test" => {
            let p: NetworkLatencyTestParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            network::latency_test(&state, p).await
        }
        // 服务器状态检测
        "server_ping" => {
            let p: ServerPingParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            network::server_ping(&state, p).await
        }
        // NBT 数据查看
        "nbt_parse" => {
            let p: NbtParseParams = serde_json::from_value(req.params)
                .map_err(|e| format!("参数解析失败: {}", e))?;
            nbt::parse(&state, p).await
        }
        _ => Err(format!("未知操作: {}", req.action)),
    }
}
