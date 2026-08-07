//! 启动配置中的连接与额外参数解析。

use crate::minecraft::version::setup::VersionSetup;

use super::super::parse_server_enter;

/// 解析服务器地址和端口，前端传入值优先于版本独立设置。
pub(super) fn resolve_server(
    setup: &VersionSetup,
    server_address: Option<String>,
    server_port: Option<u32>,
) -> (Option<String>, Option<u32>) {
    if server_address.is_some() || server_port.is_some() {
        return (server_address, server_port);
    }

    setup
        .display
        .server_enter
        .as_deref()
        .filter(|enter| !enter.is_empty())
        .map(parse_server_enter)
        .unwrap_or((None, None))
}

/// 解析 setup.ini 中的空白分隔参数，并追加单次启动参数。
pub(super) fn resolve_extra_args(
    setup: &VersionSetup,
    extra_jvm_args_override: Option<Vec<String>>,
) -> (Vec<String>, Vec<String>, Option<String>) {
    let split_args = |value: &Option<String>| {
        value
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    };

    let mut extra_jvm_args: Vec<String> = split_args(&setup.advanced.jvm_args);
    if let Some(override_args) = extra_jvm_args_override {
        extra_jvm_args.extend(override_args);
    }

    (
        extra_jvm_args,
        split_args(&setup.advanced.game_args),
        setup.advanced.run_cmd.clone().filter(|s| !s.is_empty()),
    )
}
