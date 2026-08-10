//! system manager developer 域 register（开发者模式 / 系统信息 / devtools 9 个 action）

use crate::handler;
use crate::utils::dispatcher::Dispatcher;

/// 注册 developer 域 action
pub(super) fn register(d: &mut Dispatcher) {
    // is_developer_unlocked / get_storage_dirs / get_system_info 返回非 Result，
    // handler 内用 Ok(to_value(r)?) 包装。
    d.register(
        "is_developer_unlocked",
        handler!(_state, _app, _params, {
            let r = crate::commands::system::developer::is_developer_unlocked();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "unlock_developer_mode",
        handler!(_state, _app, _params, {
            crate::commands::system::developer::unlock_developer_mode()?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "lock_developer_mode",
        handler!(_state, app, _params, {
            crate::commands::system::developer::lock_developer_mode(&app)?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "get_storage_dirs",
        handler!(_state, _app, _params, {
            let r = crate::commands::system::developer::get_storage_dirs();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "get_system_info",
        handler!(_state, _app, _params, {
            let r = crate::commands::system::developer::get_system_info();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "get_cache_stats",
        handler!(_state, _app, _params, {
            let r = crate::commands::system::developer::get_cache_stats().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );

    // 更新检测分支覆盖（2 个）：set 需开发者模式已开启（函数内 require_dev_mode 校验）
    d.register(
        "get_update_branch",
        handler!(_state, _app, _params, {
            let r = crate::commands::system::developer::get_update_branch();
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "set_update_branch",
        handler!(_state, _app, _params, {
            let branch = _params.get("branch").and_then(|v| v.as_str()).unwrap_or("");
            crate::commands::system::developer::set_update_branch(branch)?;
            Ok(serde_json::Value::Null)
        }),
    );

    // devtools 控制（3 个）：要求开发者模式已解锁且已开启，普通用户无法触发
    d.register(
        "open_devtools",
        handler!(_state, app, _params, {
            crate::commands::system::developer::open_devtools(&app)?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "close_devtools",
        handler!(_state, app, _params, {
            crate::commands::system::developer::close_devtools(&app)?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "is_devtools_open",
        handler!(_state, app, _params, {
            let r = crate::commands::system::developer::is_devtools_open(&app)?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
}
