//! system manager config 域 register（get_config_path / save_config_to_file）

use crate::handler;
use crate::utils::dispatcher::Dispatcher;

/// 注册 config 域 action
pub(super) fn register(d: &mut Dispatcher) {
    d.register(
        "get_config_path",
        handler!(_state, _app, _params, {
            let r = crate::commands::system::config::get_config_path().await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }),
    );
    d.register(
        "save_config_to_file",
        handler!(state, _app, _params, {
            crate::commands::system::config::save_config_to_file(&state).await?;
            Ok(serde_json::Value::Null)
        }),
    );
}