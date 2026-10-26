//! system manager telemetry 域 register（上报状态 / 开关 / 手动发送 3 个 action）

use serde::Deserialize;

use crate::handler;
use crate::utils::dispatcher::Dispatcher;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetEnabledParams {
    enabled: bool,
}

pub(super) fn register(d: &mut Dispatcher) {
    d.register(
        "telemetry_status",
        handler!(_state, _app, _params, { Ok(crate::telemetry::status()) }),
    );
    d.register(
        "telemetry_set_enabled",
        handler!(_state, _app, params, {
            let p: SetEnabledParams =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {}", e))?;
            crate::telemetry::set_enabled(p.enabled);
            Ok(crate::telemetry::status())
        }),
    );
    d.register(
        "telemetry_flush",
        handler!(_state, _app, _params, {
            let sent = crate::telemetry::flush().await;
            Ok(serde_json::json!({ "sent": sent }))
        }),
    );
}
