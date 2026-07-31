//! 选择器子窗口模块
//!
//! 提供通用的子窗口选择能力：前端指定模板名（+ 可选数据 + 可选 CSP），后端从
//! resources 读取 HTML 模板并注入数据，创建 Tauri 子窗口渲染。用户点击选项后
//! 通过 on_navigation 拦截 `picker-result://` 协议导航，将选中值通过 Tauri event
//! 返回前端。
//!
//! 模板由后端控制（放 `resources/templates/`），前端只传模板名，不再传入完整 HTML，
//! 防止前端注入任意 HTML/JS。
//!
//! ## URI scheme
//!
//! `picker://localhost/<id>` → 按 picker_id 查找模板名，从 resources 读取模板并注入数据返回
//! `picker://localhost/<id>/data` → 返回模板对应的实时数据 JSON（供前端定时刷新）
//! （Windows 上 Tauri 自动转为 `https://picker.localhost/<id>`）
//!
//! ## 选择协议
//!
//! HTML 中点击项导航到 `picker-result://?value=<value>`，on_navigation 拦截后：
//! 1. emit `picker-result` 事件（payload: `{ id, value }`）
//! 2. 标记 picker 已完成（避免窗口销毁时再发 `picker-cancelled`）
//! 3. 关闭窗口
//!
//! ## 取消事件
//!
//! 用户直接关窗未选择时，窗口 Destroyed 事件触发：
//! emit `picker-cancelled` 事件（payload: picker ID）
//!
//! ## CSP 策略
//!
//! 前端在 `picker-templates.ts` 中为每个模板配置 CSP 字符串，通过 `params.csp`
//! 传递给后端。后端将 CSP 存入 `PICKER_CSP_STORE`，URI scheme handler 读取后
//! 注入到 HTTP 响应头 `Content-Security-Policy` 中，限制子窗口可加载的资源范围。

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use once_cell::sync::Lazy;
use tauri::{
    http::Response, AppHandle, Builder, Emitter, Manager, Runtime, WebviewUrl,
    WebviewWindowBuilder,
};

use super::types::OpenPickerWindowParams;
use crate::log_info;

/// picker URI scheme 名称
const PICKER_SCHEME: &str = "picker";

/// 全局模板存储（picker_id → template_name）
///
/// 替代原 PICKER_HTML_STORE：不再存储前端传入的 HTML 字符串，改为存储模板名，
/// 由 URI scheme handler 从 resources 读取模板并注入数据，防止前端注入。
static PICKER_TEMPLATES: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 前端传入的模板数据存储（picker_id → data JSON）
/// port-picker 模板不需要（后端实时获取端口），redirect 等模板需要前端传入数据
static PICKER_DATA_STORE: Lazy<Mutex<HashMap<String, serde_json::Value>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// CSP 策略存储（picker_id → csp 字符串）
/// 前端通过 params.csp 传递，后端在 picker:// 响应头中注入
static PICKER_CSP_STORE: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 已完成选择的 picker ID 集合（避免 Destroyed 时重复发 picker-cancelled）
static PICKER_COMPLETED: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// picker ID 自增计数器（配合时间戳保证唯一性）
static PICKER_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 打开选择器子窗口
///
/// 生成唯一 picker ID，存储模板名、数据和 CSP，创建 WebviewWindow 加载
/// `picker://localhost/<id>`，注册 on_navigation 拦截 `picker-result://` 选择导航，
/// 注册 on_window_event 处理用户关窗取消。
pub async fn open_picker_window(
    app: AppHandle,
    params: OpenPickerWindowParams,
) -> Result<serde_json::Value, String> {
    // 生成唯一 picker ID
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let counter = PICKER_COUNTER.fetch_add(1, Ordering::SeqCst);
    let picker_id = format!("picker-{}-{}", ts, counter);

    // 存储模板名（URI scheme handler 据此读取 resources 中的模板）
    {
        let mut store = PICKER_TEMPLATES
            .lock()
            .map_err(|e| format!("template store lock error: {}", e))?;
        store.insert(picker_id.clone(), params.template.clone());
    }

    // 存储前端传入的 data（redirect 等模板通过 window.__PICKER_DATA__ 读取）
    {
        let mut data_store = PICKER_DATA_STORE
            .lock()
            .map_err(|e| format!("Data store lock error: {}", e))?;
        data_store.insert(picker_id.clone(), params.data);
    }

    // 存储前端传入的 CSP（URI scheme handler 注入到响应头）
    if let Some(csp) = params.csp.as_ref() {
        if !csp.is_empty() {
            let mut csp_store = PICKER_CSP_STORE
                .lock()
                .map_err(|e| format!("CSP store lock error: {}", e))?;
            csp_store.insert(picker_id.clone(), csp.clone());
        }
    }

    let width = params.width.unwrap_or(400.0);
    let height = params.height.unwrap_or(500.0);
    let label = picker_id.clone();

    log_info!(
        "[Picker] Opening picker window: id={}, title=\"{}\", template={}, size={}x{}, csp={}",
        picker_id,
        params.title,
        params.template,
        width,
        height,
        if params.csp.is_some() { "enabled" } else { "disabled" }
    );

    // 构造 picker:// URL（Tauri v2 在 Windows 上会转为 https://picker.localhost/）
    let url = WebviewUrl::CustomProtocol(
        tauri::Url::parse(&format!("picker://localhost/{}", picker_id))
            .map_err(|e| format!("URL parse error: {}", e))?,
    );

    let app_handle = app.clone();
    let id_for_nav = picker_id.clone();
    let id_for_close = picker_id.clone();
    let app_for_close = app.clone();

    let mut builder = WebviewWindowBuilder::new(&app, &label, url)
        .title(&params.title)
        .inner_size(width, height)
        .resizable(false)
        .center()
        .devtools(false);

    // 应用主窗口图标（与 ms-auth、plugin window 一致）
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone()).map_err(|e| e.to_string())?;
    }

    let window = builder
        .on_navigation(move |url| {
            // 拦截选择导航: picker-result://?value=XXX
            if url.scheme() == "picker-result" {
                if let Some(value) = url
                    .query_pairs()
                    .find(|(k, _)| k == "value")
                    .map(|(_, v)| v.to_string())
                {
                    // 标记已完成，阻止 Destroyed 时再发 picker-cancelled
                    if let Ok(mut completed) = PICKER_COMPLETED.lock() {
                        completed.insert(id_for_nav.clone());
                    }
                    let _ = app_handle.emit(
                        "picker-result",
                        serde_json::json!({
                            "id": &id_for_nav,
                            "value": value,
                        }),
                    );
                }
                // 清理模板存储、数据存储、CSP 存储
                cleanup_picker_stores(&id_for_nav);
                // 关闭窗口
                if let Some(win) = app_handle.get_webview_window(&id_for_nav) {
                    let _ = win.close();
                }
                false // 阻止实际导航
            } else {
                true // 允许其他导航（如初始 picker:// 加载）
            }
        })
        .build()
        .map_err(|e| format!("Failed to create picker window: {}", e))?;

    // 监听窗口关闭（用户未选择直接关窗）
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed { .. } = event {
            // 检查是否已完成选择，避免重复发取消事件
            let already_completed = PICKER_COMPLETED
                .lock()
                .map(|mut completed| completed.remove(&id_for_close))
                .unwrap_or(false);

            if !already_completed {
                let _ = app_for_close.emit("picker-cancelled", &id_for_close);
                cleanup_picker_stores(&id_for_close);
                log_info!("[Picker] Window closed without selection: {}", id_for_close);
            }
        }
    });

    Ok(serde_json::json!({ "id": picker_id }))
}

/// 清理 picker 相关存储（模板、数据、CSP）
fn cleanup_picker_stores(picker_id: &str) {
    if let Ok(mut store) = PICKER_TEMPLATES.lock() {
        store.remove(picker_id);
    }
    if let Ok(mut data_store) = PICKER_DATA_STORE.lock() {
        data_store.remove(picker_id);
    }
    if let Ok(mut csp_store) = PICKER_CSP_STORE.lock() {
        csp_store.remove(picker_id);
    }
}

/// 注册 picker:// URI scheme（在 lib.rs 中调用）
///
/// 通用化模板读取：根据模板名从 resources 读取 `templates/<name>.html`，
/// 注入数据后返回。port-picker 模板对 `/data` 请求特殊处理，返回实时端口列表。
///
/// URL 格式可能为：
///   - `picker://localhost/picker-123-0`（macOS/Linux）
///   - `https://picker.localhost/picker-123-0`（Windows）
///   - `.../picker-123-0/data`（实时数据请求）
pub fn register_picker_scheme<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.register_uri_scheme_protocol(PICKER_SCHEME, |_ctx, request| {
        let uri = request.uri().to_string();
        let is_data_request = uri.contains("/data");
        let picker_id = extract_picker_id(&uri);

        // 查找模板名
        let template_name = PICKER_TEMPLATES
            .lock()
            .ok()
            .and_then(|store| store.get(picker_id).cloned())
            .unwrap_or_default();

        // 查找 CSP
        let csp = PICKER_CSP_STORE
            .lock()
            .ok()
            .and_then(|store| store.get(picker_id).cloned());

        // port-picker 模板：/data 请求返回实时端口列表
        if template_name == "port-picker" && is_data_request {
            let ports = super::network::list_open_ports_sync();
            let json = serde_json::json!({ "ports": ports });
            return build_response(200, "application/json", serde_json::to_vec(&json).unwrap_or_default(), csp.as_deref());
        }

        // 读取模板 HTML（统一从 resources/templates/<name>.html 读取）
        let template_path = format!("templates/{}.html", template_name);
        let template = crate::resources::read_resource(&template_path)
            .unwrap_or_else(|_| "<html><body>模板不存在</body></html>".to_string());

        // 注入数据：
        // - port-picker：注入实时端口列表
        // - 其他模板：注入前端传入的 data
        let data_json = if template_name == "port-picker" {
            let ports = super::network::list_open_ports_sync();
            serde_json::json!({ "ports": ports })
        } else {
            PICKER_DATA_STORE
                .lock()
                .ok()
                .and_then(|store| store.get(picker_id).cloned())
                .unwrap_or(serde_json::json!({}))
        };

        let injection = format!("<script>window.__PICKER_DATA__ = {};</script>", data_json);
        let html = template.replace("</body>", &format!("{}</body>", injection));

        build_response(200, "text/html; charset=utf-8", html.into_bytes(), csp.as_deref())
    })
}

/// 构造响应（注入 CSP 响应头）
fn build_response(status: u16, content_type: &str, body: Vec<u8>, csp: Option<&str>) -> Response<Vec<u8>> {
    let mut builder = Response::builder()
        .status(status)
        .header("Content-Type", content_type)
        .header("Access-Control-Allow-Origin", "*");

    if let Some(csp_str) = csp {
        builder = builder.header("Content-Security-Policy", csp_str);
    }

    builder.body(body).unwrap()
}

/// 从 URI 中提取 picker_id
///
/// 查找以 `picker-` 开头的路径段作为 picker_id（跳过 `data` 等其他段）。
/// - `picker://localhost/picker-123-0` → `picker-123-0`
/// - `https://picker.localhost/picker-123-0/data` → `picker-123-0`
fn extract_picker_id(uri: &str) -> &str {
    let after_scheme = uri.split("://").nth(1).unwrap_or("");
    for segment in after_scheme.split('/') {
        let segment = segment.split('?').next().unwrap_or("");
        if segment.starts_with("picker-") {
            return segment;
        }
    }
    ""
}
