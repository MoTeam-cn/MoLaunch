//! system manager game_dir 域 register（目录 / 路径 / 内存 7 个 action）

use serde::Deserialize;

use crate::handler;
use crate::utils::dispatcher::Dispatcher;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathParams {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteTextFileParams {
    path: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetGameDirParams {
    game_dir: String,
}

/// 注册 game_dir 域 action
pub(super) fn register(d: &mut Dispatcher) {
    d.register(
        "open_game_dir",
        handler!(state, _app, _params, {
            crate::commands::system::game_dir::open_game_dir(&state).await?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "open_path",
        handler!(_state, _app, params, {
            let p: PathParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::system::game_dir::open_path(p.path).await?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "reveal_in_explorer",
        handler!(_state, _app, params, {
            let p: PathParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::system::game_dir::reveal_in_explorer(p.path).await?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "get_game_dir",
        handler!(state, _app, _params, {
            let r = crate::commands::system::game_dir::get_game_dir(&state).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "write_text_file",
        handler!(_state, _app, params, {
            let p: WriteTextFileParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::system::game_dir::write_text_file(p.path, p.content).await?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "get_system_memory",
        handler!(_state, _app, _params, {
            let r = crate::commands::system::game_dir::get_system_memory().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "set_game_dir",
        handler!(state, _app, params, {
            let p: SetGameDirParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::commands::system::game_dir::set_game_dir(&state, p.game_dir).await?;
            Ok(serde_json::Value::Null)
        }),
    );
}
