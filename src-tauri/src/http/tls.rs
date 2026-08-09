/// 仅当忽略 TLS 校验开启且目标为本地调试 host 时才允许跳过证书校验。
pub fn ignore_tls_allowed(host: &str) -> bool {
    let is_local = matches!(host, "localhost" | "127.0.0.1" | "::1");
    crate::commands::system::developer::is_ignore_tls() && is_local
}

/// 根据 trust_mode 配置 reqwest 的 TLS 信任源；host 为本地调试目标且开关开启时才跳过证书校验。
pub(super) fn configure(
    mut builder: reqwest::ClientBuilder,
    trust_mode: &str,
    host: Option<&str>,
) -> reqwest::ClientBuilder {
    if host.map(ignore_tls_allowed).unwrap_or(false) {
        return builder.danger_accept_invalid_certs(true);
    }

    let use_builtin = trust_mode.contains("builtin") || trust_mode == "all";
    let use_system = trust_mode.contains("system") || trust_mode == "all";
    let use_custom = trust_mode.contains("custom") || trust_mode == "all";
    builder = builder.tls_built_in_root_certs(use_builtin);

    if use_system {
        for cert in crate::certs::load_system_root_certificates() {
            builder = builder.add_root_certificate(cert);
        }
    }
    if use_custom {
        for cert in crate::certs::load_custom_root_certificates() {
            builder = builder.add_root_certificate(cert);
        }
    }
    builder
}
