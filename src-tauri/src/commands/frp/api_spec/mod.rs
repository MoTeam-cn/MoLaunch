//! 厂商 Open API 引擎：endpoints.json 解析 + API 调用 + frpc 配置生成
//! 子模块：registry（规格加载）/ executor（请求执行 + 统一隧道/账号 DTO）/
//! config_gen / envelope / http / jsonpath

mod executor;
mod registry;

pub mod config_gen;
pub mod envelope;
pub mod http;
pub mod jsonpath;

pub use executor::{fetch_raw_tunnel_config, fetch_tunnels, AccountInfo, TunnelInfo};
pub use registry::load_api_spec;
