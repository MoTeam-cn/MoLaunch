//! 深度链接路由注册表与分发（注册式后缀路由，仿 `utils::dispatcher::Dispatcher`）

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use tauri::{AppHandle, Emitter};
use tauri_plugin_deep_link::DeepLinkExt;

use super::handlers;
use super::request::{parse, DeeplinkRequest};
use crate::log_error;
use crate::log_info;

/// Boxed future：类型擦除的异步 handler 返回值
type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// 深度链接 handler 签名
///
/// 接收 `AppHandle`（可通过 `app.state::<AppState>()` 访问全局状态）与
/// 解析后的 [`DeeplinkRequest`]，返回 `Result<(), String>`。
pub type DeeplinkHandler = std::sync::Arc<
    dyn Fn(AppHandle, DeeplinkRequest) -> BoxFuture<Result<(), String>> + Send + Sync + 'static,
>;

/// 全局路由表：host 段 → handler
///
/// 注册时机：`init()` 在 setup 钩子中调用，先注册内置 handler 再装载全局监听，
/// 保证任何 deeplink 到达时路由表已就绪。
static ROUTER: Lazy<Mutex<HashMap<&'static str, DeeplinkHandler>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 注册一个后缀路由（如 `molaunch://run` → 传 `"run"`）
///
/// 用法：
/// ```ignore
/// deeplink::register("run", |app, req| Box::pin(async move {
///     // 处理 molaunch://run
///     Ok(())
/// }));
/// ```
pub fn register(route: &'static str, handler: DeeplinkHandler) {
    ROUTER.lock().unwrap().insert(route, handler);
}

/// 注册一个后缀路由（同步 handler 版本，自动包装为 async）
pub fn register_sync<F>(route: &'static str, handler: F)
where
    F: Fn(AppHandle, DeeplinkRequest) -> Result<(), String> + Send + Sync + 'static,
{
    let handler = std::sync::Arc::new(handler);
    register(
        route,
        std::sync::Arc::new(move |app, req| {
            let handler = handler.clone();
            Box::pin(async move { handler(app, req) })
        }),
    );
}

/// 分发一个深度链接 URL
///
/// 1. 解析 URL → [`DeeplinkRequest`]
/// 2. 按 host 段查路由表，找到则 spawn 异步执行 handler
/// 3. 未注册的路由记 warning（预留扩展点：后续可接"安装方询问"）
/// 4. 无论是否命中都 emit `deeplink://new` 事件给前端（前端可做页面跳转展示）
pub fn dispatch(app: &AppHandle, raw: &str) {
    let Some(req) = parse(raw) else {
        return;
    };
    log_info!("[Deeplink] 收到: {}", raw);

    // emit 给前端（前端监听 deeplink://new 做页面跳转）
    let _ = app.emit("deeplink://new", &req);

    // 查路由表分发
    let host = req.host.clone();
    let handler = ROUTER.lock().unwrap().get(host.as_str()).cloned();
    match handler {
        Some(h) => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = h(app.clone(), req.clone()).await {
                    log_error!("[Deeplink] handler `{}` 执行失败: {}", host, e);
                }
            });
        }
        None => {
            log_info!(
                "[Deeplink] 未注册的路由 `{}`（URL: {}），仅通知前端",
                host,
                raw
            );
        }
    }
}

/// 初始化 deeplink 模块
///
/// 在 Tauri setup 钩子中调用（lib.rs）：
/// 1. 注册内置 handler（run / install / open）
/// 2. **便携版/debug 自动注册协议**（`auto_register`，幂等）：
///    - 未注册 → 注册（dev / portable 首次启动）
///    - 已注册但指向旧路径 → 重注册到当前 exe（便携版被移动）
///    - 已注册且指向当前 exe → 跳过（安装版场景，安装器已注册）
///    - macOS 不支持运行时注册（协议由 tauri.conf.json 打包写入 Info.plist）
/// 3. 应用启动时若由 deeplink 唤醒（Windows/Linux CLI 参数 / macOS RunEvent::Opened），
///    立即分发
/// 4. 订阅插件 `deep-link://new-url` 事件，应用运行期间的链接实时分发
///
/// 返回 EventId（Copy 值，无需托管）；deep-link 插件实例由插件 setup 自行
/// `app.manage(DeepLink)`，通过 [`DeepLinkExt::deep_link()`] 访问。
pub fn init(app: &tauri::AppHandle) -> Result<tauri::EventId, Box<dyn std::error::Error>> {
    handlers::register_builtin();

    // 便携版/开发模式自动注册协议（幂等）
    #[cfg(not(target_os = "macos"))]
    {
        match super::protocol::auto_register() {
            Ok(true) => log_info!("[Deeplink] 便携版已注册 molaunch:// 协议"),
            Ok(false) => log_info!("[Deeplink] molaunch:// 协议已注册且指向当前 exe，跳过"),
            Err(e) => log_error!("[Deeplink] 注册 molaunch:// 协议失败: {}", e),
        }
    }

    // 启动时由 deeplink 唤醒的 URL
    // - Windows/Linux：OS 把 URL 作为唯一参数启动新实例，插件 handle_cli_arguments 解析
    // - macOS：冷启动 URL 经 RunEvent::Opened 注入，插件 on_event 写入 current
    if let Ok(Some(urls)) = app.deep_link().get_current() {
        for u in urls {
            dispatch(app, u.as_ref());
        }
    }

    // 应用运行期间的实时事件（Windows/Linux：single-instance 插件从新实例转发的 URL；
    // macOS：系统直接向运行中实例派发 RunEvent::Opened）
    // 闭包需要 'static，先 clone AppHandle 再 move 进闭包
    let app_for_event = app.clone();
    let event_id = app.deep_link().on_open_url(move |event| {
        for u in event.urls() {
            dispatch(&app_for_event, u.as_ref());
        }
    });
    log_info!(
        "[Deeplink] 全局监听已挂载（molaunch:// 协议），event_id={}",
        event_id
    );
    Ok(event_id)
}
