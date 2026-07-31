//! 基础选项：游戏本体、本体设置、个人信息、OptiFine

use std::path::Path;

use crate::commands::version::export::types::ExportOption;

use super::has_file_or_dir;

/// 追加基础选项：游戏本体 / 本体设置 / 个人信息 / OptiFine
pub(super) fn push(opts: &mut Vec<ExportOption>, instance_dir: &Path, rules_suffix: &str) {
    opts.push(ExportOption {
        id: "basic".into(),
        title: "游戏本体".into(),
        description: None,
        rules: None,
        show_rules: None,
        default_checked: true,
        checked: true,
        parent: None,
        enabled: false, // 必选，不可取消
        visible: true,
    });
    opts.push(ExportOption {
        id: "options".into(),
        title: "游戏本体设置".into(),
        description: Some("键位、音量、视频设置等".into()),
        rules: Some(format!("options.txt|configureddefaults/{}", rules_suffix)),
        show_rules: None,
        default_checked: true,
        checked: true,
        parent: None,
        enabled: true,
        visible: has_file_or_dir(instance_dir, &["options.txt", "configureddefaults"]),
    });
    opts.push(ExportOption {
        id: "personal_info".into(),
        title: "游戏本体个人信息".into(),
        description: Some("命令历史、已保存的快捷栏".into()),
        rules: Some(format!("hotbar.nbt|command_history.txt{}", rules_suffix)),
        show_rules: None,
        default_checked: false,
        checked: false,
        parent: None,
        enabled: true,
        visible: has_file_or_dir(instance_dir, &["hotbar.nbt", "command_history.txt"]),
    });
    opts.push(ExportOption {
        id: "optifine".into(),
        title: "OptiFine 设置".into(),
        description: None,
        rules: Some(format!("optionsof.txt|optionsshaders.txt{}", rules_suffix)),
        show_rules: None,
        default_checked: true,
        checked: true,
        parent: None,
        enabled: true,
        visible: has_file_or_dir(instance_dir, &["optionsof.txt", "optionsshaders.txt"]),
    });
}
