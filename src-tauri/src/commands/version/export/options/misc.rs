//! 杂项选项：协议文件、多人服务器列表

use std::path::Path;

use crate::commands::version::export::types::ExportOption;

use super::has_licence_file;

/// 追加杂项选项：Licence 协议 / 多人服务器列表
pub(super) fn push(opts: &mut Vec<ExportOption>, instance_dir: &Path, rules_suffix: &str) {
    opts.push(ExportOption {
        id: "licence".into(),
        title: "协议".into(),
        description: Some("Licence 文件".into()),
        rules: Some(format!("LICEN*{}", rules_suffix)),
        show_rules: None,
        default_checked: true,
        checked: true,
        parent: None,
        enabled: true,
        visible: has_licence_file(instance_dir),
    });
    opts.push(ExportOption {
        id: "servers".into(),
        title: "多人游戏服务器列表".into(),
        description: None,
        rules: Some(format!("servers.dat{}", rules_suffix)),
        show_rules: None,
        default_checked: false,
        checked: false,
        parent: None,
        enabled: true,
        visible: instance_dir.join("servers.dat").is_file(),
    });
}
