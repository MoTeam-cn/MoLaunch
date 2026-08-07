/// 根据 trust_mode 配置 reqwest 的 TLS 信任源。
pub(super) fn configure(
    mut builder: reqwest::ClientBuilder,
    trust_mode: &str,
    ignore_tls: bool,
) -> reqwest::ClientBuilder {
    if ignore_tls {
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
