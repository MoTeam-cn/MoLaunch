//! 世界相关选项：截图、结构、录像回放、单人存档（含动态子选项扫描）

use std::path::Path;

use crate::commands::version::export::types::ExportOption;

use super::scan_sub_options;

/// 追加世界相关选项：截图 / 导出结构 / 录像回放 / 单人存档
pub(super) fn push(opts: &mut Vec<ExportOption>, instance_dir: &Path, rules_suffix: &str) {
    opts.push(ExportOption {
        id: "screenshots".into(),
        title: "截图".into(),
        description: None,
        rules: Some(format!("screenshots/{}", rules_suffix)),
        show_rules: None,
        default_checked: false,
        checked: false,
        parent: None,
        enabled: true,
        visible: instance_dir.join("screenshots").is_dir(),
    });
    opts.push(ExportOption {
        id: "schematics".into(),
        title: "导出的结构".into(),
        description: Some("schematics 文件夹".into()),
        rules: Some(format!("schematics/{}", rules_suffix)),
        show_rules: None,
        default_checked: false,
        checked: false,
        parent: None,
        enabled: true,
        visible: instance_dir.join("schematics").is_dir(),
    });
    opts.push(ExportOption {
        id: "replay".into(),
        title: "录像回放".into(),
        description: Some("Replay Mod 的录像文件".into()),
        rules: Some(format!("replay_recordings/|replay_videos/{}", rules_suffix)),
        show_rules: None,
        default_checked: false,
        checked: false,
        parent: None,
        enabled: true,
        visible: instance_dir.join("replay_recordings").is_dir()
            || instance_dir.join("replay_videos").is_dir(),
    });
    let has_saves = instance_dir.join("saves").is_dir();
    opts.push(ExportOption {
        id: "saves".into(),
        title: "单人游戏存档".into(),
        description: Some("世界/地图".into()),
        rules: Some(format!("saves/{}", rules_suffix)),
        show_rules: Some("saves/".into()),
        default_checked: false,
        checked: false,
        parent: None,
        enabled: true,
        visible: has_saves,
    });
    if has_saves {
        for sub in scan_sub_options(instance_dir, &["saves"], false, true) {
            opts.push(sub);
        }
    }
}
