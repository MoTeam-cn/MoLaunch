//! 选择器子窗口模块
//! 前端传模板名（+数据+CSP），后端从 resources 读取 HTML 模板注入数据并创建 Tauri
//! 子窗口渲染。点击选项导航 `picker-result://` 被 on_navigation 拦截 emit `picker-result`
//! 事件返回前端；关窗未选则 emit `picker-cancelled`。模板由后端控制（放 `resources/templates/`）
//! 前端只传模板名防注入；URI scheme 处理见 `scheme` 子模块。

mod scheme;
mod window;

pub use scheme::register_picker_scheme;
pub use window::open_picker_window;
