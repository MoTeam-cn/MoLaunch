//! 插件子窗口创建（带域名白名单 + 数量限制）
//! `plugin_create_window` 创建独立 WebviewWindow。安全限制：manifest 须声明 `createWindow`
//! 权限 + `window_permissions` 配置；URL 域名须在 allowed_domains 白名单内（支持 `*.` 通配符
//! 前缀）；单插件最多同时 5 个窗口；窗口 label 用 `plugin-<id>-<label>` 避免与内置窗口冲突。
//! 已聚合为 `plugins_manager` IPC 入口，由 `utils::plugins_manager::dispatch` 调用。

use super::{read_plugin_manifest, WindowPermissions};
use crate::error_util::log_err;
use crate::log_info;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// 单个插件最多同时存在的窗口数
const MAX_WINDOWS_PER_PLUGIN: usize = 5;

/// 创建插件子窗口
///
/// 流程：权限校验 → 域名白名单校验 → 数量限制校验 → label 唯一性校验 → 构建 WebviewWindow
pub async fn plugin_create_window(
    app: &AppHandle,
    plugin_id: String,
    label: String,
    url: String,
    title: String,
    width: Option<f64>,
    height: Option<f64>,
) -> Result<(), String> {
    // 1. 读取 manifest
    let manifest = read_plugin_manifest(&plugin_id)?;

    // 2. 校验 createWindow 权限
    if !manifest.permissions.iter().any(|p| p == "createWindow") {
        return Err(format!(
            "Plugin {} does not have createWindow permission",
            plugin_id
        ));
    }

    // 3. 校验 window_permissions 配置
    let win_perms: &WindowPermissions = manifest
        .window_permissions
        .as_ref()
        .ok_or_else(|| format!("Plugin {} missing windowPermissions config", plugin_id))?;

    // 4. 校验 URL 协议（http/https）
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("URL must be http/https: {}", url));
    }

    // 5. 域名白名单校验
    let domain = extract_domain(&url).ok_or_else(|| format!("Invalid URL: {}", url))?;
    if !is_domain_allowed(&domain, &win_perms.allowed_domains) {
        return Err(format!("Domain not allowed: {}", domain));
    }

    // 6. 窗口 label 格式：plugin-<id>-<label>
    let window_label = format!("plugin-{}-{}", plugin_id, label);

    // 7. 窗口数量限制（统计已有的 plugin-<id>-* 窗口）
    let prefix = format!("plugin-{}-", plugin_id);
    let existing_count = app
        .webview_windows()
        .keys()
        .filter(|k| k.starts_with(&prefix))
        .count();

    if existing_count >= MAX_WINDOWS_PER_PLUGIN {
        return Err(format!(
            "Max windows ({}) reached for plugin {}",
            MAX_WINDOWS_PER_PLUGIN, plugin_id
        ));
    }

    // 8. label 唯一性校验
    if app.get_webview_window(&window_label).is_some() {
        return Err(format!("Window label already exists: {}", window_label));
    }

    // 9. 创建窗口
    let w = width.unwrap_or(win_perms.width);
    let h = height.unwrap_or(win_perms.height);

    let parsed_url = tauri::Url::parse(&url).map_err(log_err("Failed to parse window URL"))?;

    let mut builder =
        WebviewWindowBuilder::new(app, &window_label, WebviewUrl::External(parsed_url))
            .title(&title)
            .inner_size(w, h);

    // 应用默认窗口图标（来自 tauri.conf.json 的 bundle.icon 中的 PNG）
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone()).map_err(|e| e.to_string())?;
    }

    if !win_perms.resizable {
        builder = builder.resizable(false);
    }

    builder
        .build()
        .map_err(log_err("Failed to create plugin window"))?;

    log_info!("插件 {} 创建窗口 {} ({}x{})", plugin_id, window_label, w, h);

    Ok(())
}

/// 提取 URL 的域名（简单字符串解析，不依赖 url crate）
///
/// 输入：`https://www.example.com:8080/path?query#fragment`
/// 输出：`Some("www.example.com")`
fn extract_domain(url: &str) -> Option<String> {
    // 去掉协议
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))?;

    // 去掉 path/query/fragment
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = &rest[..end];

    // 去掉端口
    let host = host.split(':').next()?;

    if host.is_empty() {
        None
    } else {
        Some(host.to_lowercase())
    }
}

/// 校验域名是否在白名单内（支持 `*.` 通配符前缀）
///
/// - 精确匹配：`example.com` 匹配 `example.com`
/// - 通配符前缀：`*.example.com` 匹配 `www.example.com` / `a.b.example.com` / `example.com`
fn is_domain_allowed(domain: &str, allowed: &[String]) -> bool {
    for allowed_domain in allowed {
        let allowed_lower = allowed_domain.to_lowercase();

        // 精确匹配
        if allowed_lower == domain {
            return true;
        }

        // *.example.com 匹配子域名
        if let Some(suffix) = allowed_lower.strip_prefix("*.") {
            // 子域名匹配：domain 以 .suffix 结尾
            if domain.ends_with(&format!(".{}", suffix)) {
                return true;
            }
            // 允许 *.example.com 同时匹配 example.com 本身
            if domain == suffix {
                return true;
            }
        }
    }
    false
}
