//! Mod 相关选项：Mod 主选项 + 已禁用/整合包数据/Mod 设置/子模块数据

use std::path::Path;

use crate::commands::version::export::types::ExportOption;

use super::has_file_or_dir;

/// 追加 Mod 相关选项（mods 主选项 + has_mods 时的 9 个子选项）
pub(super) fn push(opts: &mut Vec<ExportOption>, instance_dir: &Path, rules_suffix: &str) {
    let has_mods = instance_dir.join("mods").is_dir()
        || instance_dir.join("coremods").is_dir()
        || instance_dir.join("lib").is_dir();
    opts.push(ExportOption {
        id: "mods".into(),
        title: "Mod".into(),
        description: Some("模组".into()),
        rules: Some(format!(
            "mods/|!mods/*.disabled|!mods/*.old|!mods/.connector/|coremods/|lib/{}",
            rules_suffix
        )),
        show_rules: None,
        default_checked: true,
        checked: true,
        parent: None,
        enabled: true,
        visible: has_mods,
    });

    if !has_mods {
        return;
    }

    // 已禁用的 Mod
    opts.push(ExportOption {
        id: "mods_disabled".into(),
        title: "已禁用的 Mod".into(),
        description: None,
        rules: Some(format!("mods/*.disabled|mods/*.old{}", rules_suffix)),
        show_rules: None,
        default_checked: false,
        checked: false,
        parent: Some("mods".into()),
        enabled: true,
        visible: true,
    });

    // 整合包重要数据
    opts.push(ExportOption {
        id: "pack_data".into(),
        title: "整合包重要数据".into(),
        description: Some("脚本文件、内置资源包、数据包等".into()),
        rules: Some(format!(
            "hotai/|bansoukou/|addons/|multiblocked/|modpack-update-checker/|global_packs/|global_resource_packs/|global_data_packs/|optional_data_packs/|moonlight-global-datapacks/|maps/|icon.png|mods-resourcepacks/|matmos/|resource_assorts/|resource_assorts.json|patchouli_books/|datapacks/|kubejs*/|!kubejs*/probe/|!kubejs*/exported/|!kubejs*/jsconfig.json|!kubejs*/README.txt|openloader/|worldshape/|resources/|scripts/|structures/|fontfiles/|oresources/|packmenu/|craftpresence/|pointblanks/|template*/|!template*/playerdata/|!template*/stats/{}",
            rules_suffix
        )),
        show_rules: None,
        default_checked: true,
        checked: true,
        parent: Some("mods".into()),
        enabled: true,
        visible: true,
    });

    // Mod 设置
    opts.push(ExportOption {
        id: "mod_config".into(),
        title: "Mod 设置".into(),
        description: None,
        rules: Some(format!(
            "config/|!config/jei/world/|!config/worldedit/|config/worldedit/worldedit.properties|!config/spark/|config/spark/config.json|defaultconfigs/|journeymap/config/|journeymap/server/|TrashSlotSaveState.json|customfov.txt|gg.essential.mod/|essential/|!essential/*/|!essential/*.jar*|!essential/screenshot-checksum-caches.json|!essential/microsoft_accounts.json|paragliderSettings.nbt|local/client_config.json|local/ftbl.json|local/client/sidebar_buttons.json|local/client/ftbutilities.cfg|local/client/ftblib.cfg|local/client/xencraft.cfg|liteloader.properties|default_reference.xml|CustomSkinLoader/CustomSkinLoader.json|!config/tacz/custom{}",
            rules_suffix
        )),
        show_rules: None,
        default_checked: true,
        checked: true,
        parent: Some("mods".into()),
        enabled: true,
        visible: true,
    });

    // TaCZ 枪包
    opts.push(ExportOption {
        id: "tacz".into(),
        title: "TaCZ 枪包".into(),
        description: None,
        rules: Some(format!("tacz/|config/tacz/custom/{}", rules_suffix)),
        show_rules: None,
        default_checked: true,
        checked: true,
        parent: Some("mods".into()),
        enabled: true,
        visible: instance_dir.join("tacz").is_dir() || instance_dir.join("config/tacz").is_dir(),
    });

    // 已上传的沉浸画
    opts.push(ExportOption {
        id: "immersive_paintings".into(),
        title: "已上传的沉浸画".into(),
        description: None,
        rules: Some(format!("immersive_paintings/{}", rules_suffix)),
        show_rules: None,
        default_checked: true,
        checked: true,
        parent: Some("mods".into()),
        enabled: true,
        visible: instance_dir.join("immersive_paintings").is_dir(),
    });

    // 已绘制的地图
    opts.push(ExportOption {
        id: "maps".into(),
        title: "已绘制的地图".into(),
        description: Some("地图类 Mod 为现有的存档、服务器记录的地图、路标点等".into()),
        rules: Some(format!(
            "journeymap/data/|xaero/|XaeroWaypoints/|XaeroWorldMap/{}",
            rules_suffix
        )),
        show_rules: None,
        default_checked: false,
        checked: false,
        parent: Some("mods".into()),
        enabled: true,
        visible: has_file_or_dir(
            instance_dir,
            &[
                "journeymap/data",
                "xaero",
                "XaeroWaypoints",
                "XaeroWorldMap",
            ],
        ),
    });

    // JEI 个人信息
    opts.push(ExportOption {
        id: "jei".into(),
        title: "JEI 个人信息".into(),
        description: Some("物品收藏夹等".into()),
        rules: Some(format!("config/jei/world/{}", rules_suffix)),
        show_rules: None,
        default_checked: false,
        checked: false,
        parent: Some("mods".into()),
        enabled: true,
        visible: instance_dir.join("config/jei/world").is_dir(),
    });

    // EMI 个人信息
    opts.push(ExportOption {
        id: "emi".into(),
        title: "EMI 个人信息".into(),
        description: Some("物品收藏夹、默认配方、合成历史记录等".into()),
        rules: Some(format!("emi.json{}", rules_suffix)),
        show_rules: None,
        default_checked: false,
        checked: false,
        parent: Some("mods".into()),
        enabled: true,
        visible: instance_dir.join("emi.json").is_file(),
    });

    // 帕秋莉手册个人信息
    opts.push(ExportOption {
        id: "patchouli".into(),
        title: "帕秋莉手册个人信息".into(),
        description: Some("教程书的已读记录、书签、阅读历史记录等".into()),
        rules: Some(format!("patchouli_data.json{}", rules_suffix)),
        show_rules: None,
        default_checked: false,
        checked: false,
        parent: Some("mods".into()),
        enabled: true,
        visible: instance_dir.join("patchouli_data.json").is_file(),
    });
}
