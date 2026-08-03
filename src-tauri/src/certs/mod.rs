//! TLS 证书管理模块
//!
//! 提供自定义/系统根证书加载与 certs 目录管理，信任源模式由 `AppConfig.tls.trust_mode` 控制。
//! 存储路径：`%APPDATA%/.Molaunch/certs/`（全局共享，跨启动器实例），历史便携式路径由
//! `storage::appdata::migrate_from_portable` 启动时自动迁移。

mod manage;
mod pem;

pub use manage::{
    add_custom_cert, cert_dir, list_custom_certs, load_custom_root_certificates,
    load_system_root_certificates, remove_custom_cert, CustomCertInfo,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
