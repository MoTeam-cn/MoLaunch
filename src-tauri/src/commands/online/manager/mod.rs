//! 联机模块统一分发逻辑（online_manager 的命令层实现）

mod auth_actions;
mod auth_register_login;
mod dispatcher;
mod easytier_actions;
mod easytier_download;
pub(crate) mod easytier_install;
mod lan_fake;
mod lan_probe;
mod signaling_manager;

pub(crate) use dispatcher::{
    build_device_status, login_fresh, make_client, make_storage, read_api_server_url,
    refresh_credentials,
};
pub use dispatcher::{
    dispatch, load_creds_with_auto_refresh, AuthInitResult, DeviceStatus, ServerTimeInfo,
};
pub use lan_fake::LanFakeServer;
