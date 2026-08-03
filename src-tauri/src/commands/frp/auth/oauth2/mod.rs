//! OAuth2 授权流程
//!
//! 流程（参见设计文档 §6.3）：本地启动 HTTP 服务监听 redirectPort 接收回调，
//! 浏览器跳转走 `crate::minecraft::system::shell::open_url`，
//! token 交换请求/响应解析由 flows.rs 引擎按 endpoints.json authFlows.oauth2.token 配置驱动。
//! 子模块：exchange（授权 URL 构建 + 回调 HTTP 处理）/ flow（授权流程编排）

mod exchange;
mod flow;

pub use flow::start_oauth2;
