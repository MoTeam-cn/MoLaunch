//! picker URI scheme 处理（模板读取 / 数据注入 / CSP / 响应构造）

use tauri::http::Response;
use tauri::{Builder, Runtime};

use super::window::{PICKER_CSP_STORE, PICKER_DATA_STORE, PICKER_TEMPLATES};

/// picker URI scheme 名称
const PICKER_SCHEME: &str = "picker";

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
            let ports = crate::commands::tools::network::list_open_ports_sync();
            let json = serde_json::json!({ "ports": ports });
            return build_response(
                200,
                "application/json",
                serde_json::to_vec(&json).unwrap_or_default(),
                csp.as_deref(),
            );
        }

        // 读取模板 HTML（统一从 resources/templates/<name>.html 读取）
        let template_path = format!("templates/{}.html", template_name);
        let template = crate::resources::read_resource(&template_path)
            .unwrap_or_else(|_| "<html><body>模板不存在</body></html>".to_string());

        // 注入数据：
        // - port-picker：注入实时端口列表
        // - 其他模板：注入前端传入的 data
        let data_json = if template_name == "port-picker" {
            let ports = crate::commands::tools::network::list_open_ports_sync();
            serde_json::json!({ "ports": ports })
        } else {
            PICKER_DATA_STORE
                .lock()
                .ok()
                .and_then(|store| store.get(picker_id).cloned())
                .unwrap_or(serde_json::json!({}))
        };

        // tutorial-* 模板：使用 base-help.html 作为基础模板，通过占位符替换注入内容
        // 不走 __PICKER_DATA__ 注入路径，直接在后端完成字符串替换形成完整 HTML，
        // 避免 JS 运行时时序问题（注入脚本与原始脚本的执行顺序导致读取到 undefined）
        if template_name.starts_with("tutorial-") {
            let content = template; // 原始读取的内容文件（content-only HTML）
            let base = crate::resources::read_resource("templates/base-help.html")
                .unwrap_or_else(|_| "<html><body>模板不存在</body></html>".to_string());
            // 从 data 中提取 title（open_picker_window 已注入 title 字段）
            let title = data_json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("帮助文档");
            // 占位符替换：形成完整 HTML，无需 JS 运行时注入
            let html = base
                .replace("{{__TITLE__}}", title)
                .replace("{{__CONTENT__}}", &content);
            return build_response(
                200,
                "text/html; charset=utf-8",
                html.into_bytes(),
                csp.as_deref(),
            );
        }

        // 注入依赖库（markdown 模板需要 marked/dompurify，qrcode 模板需要 qrcode）
        // tutorial-* 模板使用 base-help.html 硬编码 HTML，无需注入依赖库
        // 直接内联嵌入（picker 子窗口跨源 script 加载受 CSP 限制）
        let lib_script = match template_name.as_str() {
            "markdown" => inject_libs(&["view/marked.min.js", "view/dompurify.min.js"]),
            "qrcode" => inject_libs(&["view/qrcode.min.js"]),
            _ => None,
        };

        // 防 </script> 逃逸：闭合标签转为 JSON 合法转义（JS 解析后还原，HTML 解析器不再视为闭合）
        let data_json_safe = serde_json::to_string(&data_json)
            .unwrap_or_else(|_| "{}".to_string())
            .replace("</script", "<\\/script")
            .replace("<!--", "<\\!--");

        let mut injection = String::new();
        if let Some(lib) = &lib_script {
            injection.push_str(lib);
        }
        injection.push_str(&format!(
            "<script>window.__PICKER_DATA__ = {};</script>",
            data_json_safe
        ));
        let html = template.replace("</body>", &format!("{}</body>", injection));

        build_response(
            200,
            "text/html; charset=utf-8",
            html.into_bytes(),
            csp.as_deref(),
        )
    })
}

/// 读取多个 JS 库并内联为 <script> 串
fn inject_libs(paths: &[&str]) -> Option<String> {
    let mut html = String::new();
    for path in paths {
        if let Some(js) = crate::resources::read_resource_bytes(path)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
        {
            html.push_str(&format!("<script>{}</script>", js));
        }
    }
    (!html.is_empty()).then_some(html)
}

/// 构造响应（注入 CSP 响应头）
fn build_response(
    status: u16,
    content_type: &str,
    body: Vec<u8>,
    csp: Option<&str>,
) -> Response<Vec<u8>> {
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
