use serde::{Deserialize, Serialize};

/// 选择器子窗口请求参数
///
/// `template` 指定后端 resources 中的模板名（如 "port-picker"），由 URI scheme
/// handler 读取模板并注入 `data` 后渲染。不再由前端传入完整 HTML，防止注入。
///
/// `csp` 为可选的 Content-Security-Policy 策略字符串，由前端按模板类型配置，
/// 后端在 picker:// 响应头中注入，限制子窗口可加载的资源范围。
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPickerWindowParams {
    pub title: String,
    pub template: String,
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    /// Content-Security-Policy 策略字符串（前端配置，通过 IPC 传递）
    /// 后端在创建 picker:// 响应时注入到 HTTP 响应头中
    #[serde(default)]
    pub csp: Option<String>,
}
