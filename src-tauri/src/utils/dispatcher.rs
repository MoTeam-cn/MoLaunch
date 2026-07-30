//! 通用 action 分发器
//!
//! 提供 `Dispatcher` 结构，把多个 action 注册到统一入口，替代冗长的 match 语句。
//! 配合 `handler!` 宏自动包装 `Box::pin(async move { ... })`。

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::state::AppState;

/// 统一请求体（meta_manager / tools_manager 共用）
///
/// 与原 `MetaRequest` / `ToolsRequest` 字段完全一致，可直接替换。
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionRequest {
    pub action: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Boxed future：类型擦除的异步返回值
type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// 统一的 handler 函数签名
///
/// 使用 owned 参数（`AppState` / `AppHandle`）而非引用，避免 HRTB 复杂性；
/// `AppState` 与 `AppHandle` 内部均为 `Arc`，clone 廉价。
pub type Handler = std::sync::Arc<
    dyn Fn(AppState, AppHandle, serde_json::Value) -> BoxFuture<Result<serde_json::Value, String>>
        + Send + Sync
        + 'static,
>;

/// 分发器：注册 action → handler 映射
///
/// 每个 manager 创建自己的 `Dispatcher` 实例，用 `once_cell::sync::Lazy` 作为
/// 全局静态变量，启动时注册所有 actions。
pub struct Dispatcher {
    handlers: HashMap<&'static str, Handler>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// 注册 action → handler 映射
    ///
    /// 推荐配合 `handler!` 宏使用，自动处理 `Box::pin(async move { ... })` 包装。
    pub fn register<F, Fut>(&mut self, action: &'static str, handler: F)
    where
        F: Fn(AppState, AppHandle, serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<serde_json::Value, String>> + Send + 'static,
    {
        let wrapped: Handler = std::sync::Arc::new(move |state, app, params| {
            Box::pin(handler(state, app, params))
        });
        self.handlers.insert(action, wrapped);
    }

    /// 按 `req.action` 分发到对应 handler
    pub async fn dispatch(
        &self,
        state: AppState,
        app: AppHandle,
        req: ActionRequest,
    ) -> Result<serde_json::Value, String> {
        match self.handlers.get(req.action.as_str()) {
            Some(handler) => handler(state, app, req.params).await,
            None => Err(format!("未知操作: {}", req.action)),
        }
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// handler! 宏：简化 handler 注册
///
/// 自动处理 `Box::pin(async move { ... })` 包装，让注册代码更简洁。
#[macro_export]
macro_rules! handler {
    ($state:ident, $app:ident, $params:ident, $body:block) => {
        move |$state: $crate::state::AppState,
              $app: tauri::AppHandle,
              $params: serde_json::Value| {
            Box::pin(async move $body)
        }
    };
}
