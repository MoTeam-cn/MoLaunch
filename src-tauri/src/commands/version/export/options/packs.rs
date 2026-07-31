//! 资源包与光影包选项（含动态子选项扫描）

use std::path::Path;

use crate::commands::version::export::types::ExportOption;

use super::scan_sub_options;

/// 追加资源包 + 光影包选项（含动态子选项扫描）
pub(super) fn push(opts: &mut Vec<ExportOption>, instance_dir: &Path, rules_suffix: &str) {
    let has_rp = instance_dir.join("resourcepacks").is_dir()
        || instance_dir.join("texturepacks").is_dir();
    opts.push(ExportOption {
        id: "resourcepacks".into(),
        title: "资源包".into(),
        description: Some("纹理包/材质包".into()),
        rules: Some(format!("resourcepacks/|texturepacks/{}", rules_suffix)),
        show_rules: Some("resourcepacks/|texturepacks/".into()),
        default_checked: true,
        checked: true,
        parent: None,
        enabled: true,
        visible: has_rp,
    });
    if has_rp {
        for sub in scan_sub_options(instance_dir, &["resourcepacks", "texturepacks"], true, true) {
            opts.push(sub);
        }
    }

    let has_sp = instance_dir.join("shaderpacks").is_dir();
    opts.push(ExportOption {
        id: "shaderpacks".into(),
        title: "光影包".into(),
        description: None,
        rules: Some(format!("shaderpacks/{}", rules_suffix)),
        show_rules: Some("shaderpacks/".into()),
        default_checked: true,
        checked: true,
        parent: None,
        enabled: true,
        visible: has_sp,
    });
    if has_sp {
        for sub in scan_sub_options(instance_dir, &["shaderpacks"], true, true) {
            opts.push(sub);
        }
    }
}
