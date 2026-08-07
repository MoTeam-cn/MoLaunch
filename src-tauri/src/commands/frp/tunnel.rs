//! 隧道管理：CRUD、持久化与 frpc TOML 配置生成。

mod config;
mod crud;
mod import;
mod params;

pub use config::generate_config;
pub use crud::{create_tunnel, delete_tunnel, list_tunnels, update_tunnel};
pub use import::{import_frpc_config, ImportedFrpcConfig};
pub use params::{CreateTunnelParams, TunnelIdParams, UpdateTunnelParams};
