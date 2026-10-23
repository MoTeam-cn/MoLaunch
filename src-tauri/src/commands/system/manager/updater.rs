//! system manager updater 域 register（更新检查 / 下载安装 / 待安装应用 / 待安装状态 5 个 action）

use crate::handler;
use crate::utils::dispatcher::Dispatcher;

/// 注册 updater 域 action
pub(super) fn register(d: &mut Dispatcher) {
    // Windows 便携版自实现 + macOS/Linux 转发官方 plugin
    d.register(
        "check_update",
        handler!(state, app, _params, {
            let r = crate::commands::system::updater::check_update(&state, &app).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "download_and_install_update",
        handler!(state, app, params, {
            let p: crate::commands::system::updater::UpdateInfo =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::system::updater::download_and_install(&app, &state, p).await?;
            Ok(serde_json::Value::Null)
        }),
    );

    // Windows 便携版后台静默下载新版本到 appdata/last.exe
    d.register(
        "download_update_to_appdata",
        handler!(state, _app, params, {
            let p: crate::commands::system::updater::UpdateInfo =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let downloaded =
                crate::commands::system::updater::download_update_to_appdata(&state, p).await?;
            serde_json::to_value(downloaded).map_err(|e| e.to_string())
        }),
    );

    // 退出时 / 更新抽屉「立即重启」时应用待安装更新（last.exe → 替换主 exe）
    //
    // params.relaunch=true：替换完成后由 updater.exe 自动拉起新版本（立即重启）
    // 省略或 false：退出时静默替换，下次启动即为新版本
    d.register(
        "apply_pending_update",
        handler!(_state, app, params, {
            let relaunch = params
                .get("relaunch")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let has_update =
                crate::commands::system::updater::apply_pending_update(&app, relaunch).await?;
            serde_json::to_value(has_update).map_err(|e| e.to_string())
        }),
    );

    // 查询是否已有预下载完成、可直接重启安装的更新包
    d.register(
        "pending_update_status",
        handler!(_state, _app, _params, {
            let status = crate::commands::system::updater::pending_update_status().await?;
            serde_json::to_value(status).map_err(|e| e.to_string())
        }),
    );
}
