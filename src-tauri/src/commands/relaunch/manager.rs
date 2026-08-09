//! 重启快照加解密分发（encrypt / decrypt 两个 action，复用 SDK 加密封装）

use once_cell::sync::Lazy;
use tauri::AppHandle;

use crate::handler;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};
use crate::utils::sdk_crypto;

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    d.register(
        "encrypt",
        handler!(state, _app, params, {
            let data = params
                .get("data")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "缺少 data 参数".to_string())?;
            let enc = sdk_crypto::encrypt_with_sdk(&state.sdk, data, "重启快照").await?;
            Ok(serde_json::Value::String(enc))
        }),
    );

    d.register(
        "decrypt",
        handler!(state, _app, params, {
            let data = params
                .get("data")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "缺少 data 参数".to_string())?;
            let plain = sdk_crypto::decrypt_with_sdk(&state.sdk, data, "重启快照").await?;
            Ok(serde_json::Value::String(plain))
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
