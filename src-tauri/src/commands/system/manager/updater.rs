//! system manager updater 域 register（更新检查 / 下载安装 / 待安装应用 4 个 action）

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
        handler!(_state, app, params, {
            let p: crate::commands::system::updater::UpdateInfo =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::system::updater::download_and_install(&app, p).await?;
            Ok(serde_json::Value::Null)
        }),
    );

    // Windows 便携版后台静默下载新版本到 appdata/last.exe
    d.register(
        "download_update_to_appdata",
        handler!(_state, _app, params, {
            let p: crate::commands::system::updater::UpdateInfo =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            let downloaded = crate::commands::system::updater::download_update_to_appdata(p).await?;
            serde_json::to_value(downloaded).map_err(|e| e.to_string())
        }),
    );

    // 退出时检查并应用待安装更新（last.exe → 替换主 exe）
    d.register(
        "apply_pending_update",
        handler!(_state, app, _params, {
            let has_update = crate::commands::system::updater::apply_pending_update(&app).await?;
            serde_json::to_value(has_update).map_err(|e| e.to_string())
        }),
    );
}