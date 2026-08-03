//! 工具模块统一分发逻辑（tools_manager 的实现）
//! 25+ 个 tools action 在 `once_cell::sync::Lazy` 初始化时按类别注册。

use once_cell::sync::Lazy;
use tauri::AppHandle;

use super::types::*;
use super::{
    archive, cleanup, crash_analyzer, data_export, download, filename, memory, mod_tools, nbt,
    network, picker_window, resourcepack, screenshot, version_json,
};
use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    // 外部下载
    d.register(
        "download_file",
        handler!(state, _app, params, {
            let p: DownloadFileParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            download::download_file(&state, p).await
        }),
    );
    d.register(
        "get_download_dir",
        handler!(state, _app, _params, {
            download::get_download_dir(&state).await
        }),
    );
    d.register(
        "list_downloads",
        handler!(state, _app, _params, {
            download::list_downloads(&state).await
        }),
    );
    d.register(
        "delete_download",
        handler!(state, _app, params, {
            let p: DeleteDownloadParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            download::delete_download(&state, p).await
        }),
    );

    // 文件名获取
    d.register(
        "fetch_filename",
        handler!(_state, _app, params, {
            let p: FetchFilenameParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            filename::fetch_filename(p).await
        }),
    );

    // 清理游戏垃圾
    d.register(
        "cleanup_scan",
        handler!(state, _app, _params, { cleanup::scan(&state).await }),
    );
    d.register(
        "cleanup_execute",
        handler!(_state, _app, params, {
            let p: CleanupExecuteParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            cleanup::execute(p).await
        }),
    );

    // 内存优化
    d.register(
        "memory_optimize",
        handler!(_state, _app, params, {
            let p: MemoryOptimizeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            memory::optimize(p).await
        }),
    );

    // Mod 依赖检测
    d.register(
        "mod_dependency_check",
        handler!(state, _app, params, {
            let p: ModDependencyCheckParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            mod_tools::mod_dependency_check(&state, p).await
        }),
    );
    // Mod 去重扫描
    d.register(
        "mod_dedup_scan",
        handler!(state, _app, params, {
            let p: ModDedupScanParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            mod_tools::mod_dedup_scan(&state, p).await
        }),
    );

    // 启动器数据导出
    d.register(
        "export_launcher_data",
        handler!(state, _app, params, {
            let p: ExportLauncherDataParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            data_export::export_launcher_data(&state, p).await
        }),
    );

    // 崩溃日志分析
    d.register(
        "crash_analyze",
        handler!(state, _app, params, {
            let p: CrashAnalyzeParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crash_analyzer::analyze(&state, p).await
        }),
    );

    // 截图管理
    d.register(
        "screenshot_list",
        handler!(state, _app, params, {
            let p: ScreenshotListParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            screenshot::list(&state, p).await
        }),
    );
    d.register(
        "screenshot_delete",
        handler!(state, _app, params, {
            let p: ScreenshotDeleteParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            screenshot::delete(&state, p).await
        }),
    );

    // 资源包管理
    d.register(
        "resourcepack_list",
        handler!(state, _app, params, {
            let p: ResourcePackListParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            resourcepack::list(&state, p).await
        }),
    );
    d.register(
        "resourcepack_convert",
        handler!(state, _app, params, {
            let p: ResourcePackConvertParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            resourcepack::convert(&state, p).await
        }),
    );

    // 版本 JSON 读写
    d.register(
        "version_json_read",
        handler!(state, _app, params, {
            let p: VersionJsonReadParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            version_json::read(&state, p).await
        }),
    );
    d.register(
        "version_json_save",
        handler!(state, _app, params, {
            let p: VersionJsonSaveParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            version_json::save(&state, p).await
        }),
    );

    // 存档管理
    d.register(
        "archive_list",
        handler!(state, _app, params, {
            let p: ArchiveListParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            archive::list(&state, p).await
        }),
    );
    d.register(
        "archive_backup",
        handler!(state, _app, params, {
            let p: ArchiveBackupParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            archive::backup(&state, p).await
        }),
    );
    d.register(
        "archive_restore",
        handler!(state, _app, params, {
            let p: ArchiveRestoreParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            archive::restore(&state, p).await
        }),
    );
    d.register(
        "extract_save_seed",
        handler!(state, _app, params, {
            let p: ExtractSaveSeedParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            archive::extract_save_seed(&state, p).await
        }),
    );

    // 网络延迟测试
    d.register(
        "network_latency_test",
        handler!(state, _app, params, {
            let p: NetworkLatencyTestParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            network::latency_test(&state, p).await
        }),
    );
    // 服务器状态检测
    d.register(
        "server_ping",
        handler!(state, _app, params, {
            let p: ServerPingParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            network::server_ping(&state, p).await
        }),
    );
    // TCP 端口连通性检测（Frp 等非 MC 协议服务）
    d.register(
        "tcp_check",
        handler!(state, _app, params, {
            let p: TcpCheckParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            network::tcp_check(&state, p).await
        }),
    );

    // 列出本机监听端口（供 Frp 内网端口选择）
    d.register(
        "list_open_ports",
        handler!(state, _app, _params, {
            network::list_open_ports(&state).await
        }),
    );

    // 选择器子窗口（通用 HTML 渲染 + on_navigation 选择回调）
    d.register(
        "open_picker_window",
        handler!(_state, app, params, {
            let p: OpenPickerWindowParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            picker_window::open_picker_window(app, p).await
        }),
    );

    // NBT 数据查看
    d.register(
        "nbt_parse",
        handler!(state, _app, params, {
            let p: NbtParseParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            nbt::parse(&state, p).await
        }),
    );

    d
});

/// 工具 action 分发入口
pub(crate) async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}
