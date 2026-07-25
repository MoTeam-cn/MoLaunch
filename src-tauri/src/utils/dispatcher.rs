//! 通用 action 分发器
//!
//! 提供 `Dispatcher` 结构，用于把多个 action 注册到统一的入口，
//! 替代每个 manager 模块冗长的 match 语句。
//!
//! ## 设计
//!
//! - `ActionRequest`：统一请求体（meta_manager / tools_manager 共用）
//! - `Dispatcher`：注册 action → handler 映射，dispatch 时按 action 查找调用
//! - `Handler`：统一签名 `Fn(AppState, AppHandle, Value) -> BoxFuture<Result<Value, String>>`
//!   使用 owned 参数（`AppState` / `AppHandle`）而非引用，避免 HRTB 复杂性
//!   （`AppState` 派生了 `Clone`，所有字段均为 `Arc<...>`，克隆开销极低）
//! - `handler!` 宏：简化 handler 注册，自动包装 `Box::pin(async move { ... })`
//!
//! ## 用法
//!
//! ```ignore
//! use once_cell::sync::Lazy;
//! use crate::utils::dispatcher::{Dispatcher, ActionRequest, handler};
//!
//! static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
//!     let mut d = Dispatcher::new();
//!     d.register("my_action", handler!(state, _app, params, {
//!         let p: MyParams = serde_json::from_value(params)
//!             .map_err(|e| format!("参数解析失败: {}", e))?;
//!         let r = my_func(&state, p.field).await?;
//!         serde_json::to_value(r).map_err(|e| e.to_string())
//!     }));
//!     d
//! });
//!
//! pub async fn dispatch(state: AppState, app: AppHandle, req: ActionRequest)
//!     -> Result<serde_json::Value, String>
//! {
//!     DISPATCHER.dispatch(state, app, req).await
//! }
//! ```

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
/// 使用 owned 参数（`AppState` / `AppHandle`）而非引用：
/// - 避免 HRTB（for<'a>）的编译器复杂性
/// - `AppState` 已派生 `Clone`，所有字段均为 `Arc<...>`，克隆开销极低
///   （仅 N 个原子计数自增，N 为字段数 ≈ 12）
/// - `AppHandle` 内部已是 `Arc`，clone 同样廉价
/// - handler 返回 `'static` future，可安全存入 `HashMap`
pub type Handler = std::sync::Arc<
    dyn Fn(AppState, AppHandle, serde_json::Value) -> BoxFuture<Result<serde_json::Value, String>>
        + Send + Sync
        + 'static,
>;

/// 分发器：注册 action → handler 映射
///
/// 每个 manager（meta_manager / tools_manager）创建自己的 `Dispatcher` 实例，
/// 用 `once_cell::sync::Lazy` 作为全局静态变量，启动时注册所有 actions。
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
///
/// ## 用法
///
/// ```ignore
/// d.register("my_action", handler!(state, _app, params, {
///     let p: MyParams = serde_json::from_value(params)
///         .map_err(|e| format!("参数解析失败: {}", e))?;
///     let r = my_func(&state, p.field).await?;
///     serde_json::to_value(r).map_err(|e| e.to_string())
/// }));
/// ```
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
