//! 选择器子窗口打开逻辑（窗口构建 / 导航拦截 / 关窗取消事件）

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use once_cell::sync::Lazy;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use super::super::types::OpenPickerWindowParams;
use crate::log_info;

/// 全局模板存储（picker_id → template_name）
///
/// 替代原 PICKER_HTML_STORE：不再存储前端传入的 HTML 字符串，改为存储模板名，
/// 由 URI scheme handler 从 resources 读取模板并注入数据，防止前端注入。
pub(super) static PICKER_TEMPLATES: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 前端传入的模板数据存储（picker_id → data JSON）
/// port-picker 模板不需要（后端实时获取端口），redirect 等模板需要前端传入数据
pub(super) static PICKER_DATA_STORE: Lazy<Mutex<HashMap<String, serde_json::Value>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// CSP 策略存储（picker_id → csp 字符串）
/// 前端通过 params.csp 传递，后端在 picker:// 响应头中注入
pub(super) static PICKER_CSP_STORE: Lazy<Mutex<HashMap<String, String>>> =
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
    // 同时注入窗口标题（base-help 等模板从 data.title 读取标题栏文字）
    {
        let mut data_store = PICKER_DATA_STORE
            .lock()
            .map_err(|e| format!("Data store lock error: {}", e))?;
        let mut enriched = params.data;
        if let Some(obj) = enriched.as_object_mut() {
            obj.insert(
                "title".to_string(),
                serde_json::Value::String(params.title.clone()),
            );
        }
        data_store.insert(picker_id.clone(), enriched);
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
        if params.csp.is_some() {
            "enabled"
        } else {
            "disabled"
        }
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
        if let tauri::WindowEvent::Destroyed = event {
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