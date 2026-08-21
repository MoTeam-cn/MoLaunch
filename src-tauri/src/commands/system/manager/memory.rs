//! system manager memory 域 register（内存订阅/退订 2 个 action）

use crate::handler;
use crate::utils::dispatcher::Dispatcher;

pub(super) fn register(d: &mut Dispatcher) {
    d.register(
        "memory_subscribe",
        handler!(state, app, _params, {
            crate::commands::system::memory_push::memory_subscribe(&state, &app).await?;
            Ok(serde_json::Value::Null)
        }),
    );
    d.register(
        "memory_unsubscribe",
        handler!(state, _app, _params, {
            crate::commands::system::memory_push::memory_unsubscribe(&state).await?;
            Ok(serde_json::Value::Null)
        }),
    );
}
