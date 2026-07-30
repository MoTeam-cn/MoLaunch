//! `res://` 自定义 URI scheme 协议
//!
//! 提供前端访问后端嵌入资源（如 WASM 文件）的能力。
//! 协议格式：`res://web-common/{type}/{filename}`
//!   - Windows/Android: `https://res.localhost/web-common/{type}/{filename}`
//!   - macOS/Linux: `res://localhost/web-common/{type}/{filename}`
//!
//! 资源来自 `resources.rs`（编译期嵌入），URL 路径白名单校验防路径遍历，
//! 响应附带 CORS 头允许 Worker fetch，WASM 返回 `application/wasm` MIME
//! 支持 `WebAssembly.compileStreaming`。

use std::path::{Component, Path, PathBuf};
use tauri::{http::Response, Builder, Runtime};

/// 协议名（前端 URL 中的 scheme 部分）
pub const RES_SCHEME: &str = "res";

/// 资源根路径前缀（URL 中固定为 `/web-common/`）
pub const RES_ROOT: &str = "web-common";

/// 注册 `res://` 自定义 URI scheme（在 `lib.rs` 中调用）
pub fn register_res_scheme<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.register_uri_scheme_protocol(RES_SCHEME, |_ctx, request| {
        handle_res_request(&request)
    })
}

/// 处理 `res://` 协议请求
fn handle_res_request(request: &tauri::http::Request<Vec<u8>>) -> Response<Vec<u8>> {
    let uri = request.uri().to_string();
    crate::log_info!("[ResScheme] 请求: {}", uri);

    // 解析路径：URL 形如 https://res.localhost/web-common/wasm/cubiomes.wasm
    // 或 res://localhost/web-common/wasm/cubiomes.wasm
    let path = match parse_res_path(&uri) {
        Some(p) => p,
        None => {
            crate::log_warn!("[ResScheme] 无效的请求路径: {}", uri);
            return empty_response(403);
        }
    };

    // 校验路径安全性（防止 `..` 路径遍历）
    if !is_safe_path(&path) {
        crate::log_warn!("[ResScheme] 路径遍历攻击拒绝: {}", path);
        return empty_response(403);
    }

    // 将 URL 路径映射到 resources/ 下的相对路径
    // 例：web-common/wasm/cubiomes.wasm → wasm/cubiomes.wasm
    let resource_path = match map_to_resource_path(&path) {
        Some(p) => p,
        None => {
            crate::log_warn!("[ResScheme] 不支持的资源路径: {}", path);
            return empty_response(404);
        }
    };

    // 读取嵌入资源
    match crate::resources::read_resource_bytes(&resource_path) {
        Ok(bytes) => {
            let mime = guess_mime(&resource_path);
            crate::log_info!(
                "[ResScheme] 命中资源: {} ({} 字节, {})",
                resource_path,
                bytes.len(),
                mime
            );
            Response::builder()
                .status(200)
                .header("Content-Type", mime)
                .header("Cache-Control", "public, max-age=86400")
                .header("Access-Control-Allow-Origin", "*")
                .body::<Vec<u8>>(bytes)
                .unwrap()
        }
        Err(e) => {
            crate::log_warn!(
                "[ResScheme] 资源不存在或未注册: {} ({})",
                resource_path,
                e
            );
            empty_response(404)
        }
    }
}

/// 从 URL 中提取 res 资源路径（含 RES_ROOT 前缀）
///
/// 输入：`https://res.localhost/web-common/wasm/cubiomes.wasm`
/// 输出：`web-common/wasm/cubiomes.wasm`
fn parse_res_path(uri: &str) -> Option<String> {
    // 查找 "/web-common/" 在 URI 中的位置
    let root_marker = format!("/{}/", RES_ROOT);
    let idx = uri.find(&root_marker)?;
    // 从 "/web-common/" 的起始位置（含前导 '/'）截取到 query string 之前
    let rest = &uri[idx..];
    let path = rest.split('?').next()?;
    // 去掉前导 '/'
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return None;
    }
    Some(path.to_string())
}

/// 校验路径安全性：无 `..`、无绝对路径、无 Windows 盘符
fn is_safe_path(path: &str) -> bool {
    let p = Path::new(path);
    for comp in p.components() {
        match comp {
            Component::ParentDir => return false, // 禁止 ..
            Component::RootDir => return false,   // 禁止绝对路径
            Component::Prefix(_) => return false, // 禁止 Windows 盘符
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    true
}

/// 将 URL 路径映射到 resources/ 下的相对路径
///
/// `web-common/wasm/cubiomes.wasm` → `wasm/cubiomes.wasm`
/// `web-common/wasm/sub/foo.wasm` → `wasm/sub/foo.wasm`
fn map_to_resource_path(url_path: &str) -> Option<String> {
    let p = Path::new(url_path);
    let mut iter = p.components();
    // 第一个组件必须是 RES_ROOT
    match iter.next()? {
        Component::Normal(first) if first.to_str() == Some(RES_ROOT) => {}
        _ => return None,
    }
    // 拼接剩余部分
    let remaining: PathBuf = iter.filter_map(|c| match c {
        Component::Normal(s) => Some(s),
        _ => None,
    }).collect();
    remaining.to_str().map(|s| s.replace('\\', "/"))
}

/// 根据文件扩展名猜测 MIME 类型
fn guess_mime(path: &str) -> &'static str {
    let p = path.to_lowercase();
    if p.ends_with(".wasm") {
        "application/wasm"
    } else if p.ends_with(".js") {
        "application/javascript"
    } else if p.ends_with(".json") {
        "application/json"
    } else if p.ends_with(".png") {
        "image/png"
    } else if p.ends_with(".jpg") || p.ends_with(".jpeg") {
        "image/jpeg"
    } else if p.ends_with(".svg") {
        "image/svg+xml"
    } else if p.ends_with(".css") {
        "text/css"
    } else if p.ends_with(".html") {
        "text/html"
    } else {
        "application/octet-stream"
    }
}

/// 构造空的错误响应（附带 CORS 头）
fn empty_response(status: u16) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .body::<Vec<u8>>(Vec::new())
        .unwrap()
}
