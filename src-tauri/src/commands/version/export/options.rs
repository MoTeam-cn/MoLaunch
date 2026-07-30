//! 导出选项定义 + 动态子选项扫描
//!
//! 提供 ~20 个静态选项 + 资源包/存档/光影包
//! 的动态子选项扫描。选项可见性根据实例目录实际文件决定。

use std::path::Path;

use super::types::ExportOption;

/// 全局排除规则（附加在所有规则末尾，避免打包无用文件）
pub const GLOBAL_EXCLUDES: &[&str] = &[
    "!*.log",
    "!*.dat_old",
    "!*.BakaCoreInfo",
    "!hmclversion.cfg",
    "!log4j2.xml",
];

/// 子选项黑名单（资源包/存档/光影包扫描时跳过这些名称）
const SUB_OPTION_BLACKLIST: &[&str] = &["Quark Programmer Art.zip", "+ EuphoriaPatches_"];

/// 构建所有导出选项（静态 + 动态子选项）
pub fn build_all_options(instance_dir: &Path) -> Vec<ExportOption> {
    let mut opts = Vec::new();
    let rules_suffix = format!("|{}", GLOBAL_EXCLUDES.join("|"));
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

    if has_mods {
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
                &["journeymap/data", "xaero", "XaeroWaypoints", "XaeroWorldMap"],
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

    opts
}

/// 扫描子文件夹/压缩包，生成动态子选项（用于资源包/存档/光影包）
fn scan_sub_options(
    instance_dir: &Path,
    folders: &[&str],
    accept_compressed: bool,
    accept_folder: bool,
) -> Vec<ExportOption> {
    let mut result = Vec::new();
    let parent_id = folders[0]; // 用第一个文件夹名作为 parent id 后缀

    for folder in folders {
        let target = instance_dir.join(folder);
        if !target.is_dir() {
            continue;
        }

        let entries = match std::fs::read_dir(&target) {
            Ok(e) => e.flatten().collect::<Vec<_>>(),
            Err(_) => continue,
        };

        // 压缩包文件
        if accept_compressed {
            for entry in &entries {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let file_name = entry.file_name();
                let name = match file_name.to_str() {
                    Some(n) => n,
                    None => continue,
                };
                if !name.ends_with(".zip") && !name.ends_with(".rar") {
                    continue;
                }
                if SUB_OPTION_BLACKLIST.iter().any(|b| name.contains(b)) {
                    continue;
                }
                let rule = format!("{}/{}", folder, name);
                result.push(ExportOption {
                    id: format!("{}_file_{}", parent_id, name),
                    title: name.to_string(),
                    description: None,
                    rules: Some(escape_glob_chars(&rule)),
                    show_rules: None,
                    default_checked: true,
                    checked: true,
                    parent: Some(parent_id.to_string()),
                    enabled: true,
                    visible: true,
                });
            }
        }

        // 子文件夹
        if accept_folder {
            let mut subdirs: Vec<_> = entries
                .into_iter()
                .filter(|e| e.path().is_dir())
                .collect();
            subdirs.sort_by_key(|e| {
                e.metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .map(|t| t)
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            });
            for entry in subdirs.into_iter().rev() {
                let path = entry.path();
                let file_name = entry.file_name();
                let name = match file_name.to_str() {
                    Some(n) => n,
                    None => continue,
                };
                if SUB_OPTION_BLACKLIST.iter().any(|b| name.contains(b)) {
                    continue;
                }
                // 跳过空文件夹
                if std::fs::read_dir(&path)
                    .map(|mut r| r.next().is_none())
                    .unwrap_or(true)
                {
                    continue;
                }
                let rule = format!("{}/{}/", folder, name);
                let description = if parent_id == "saves" {
                    entry
                        .metadata()
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| {
                            let secs = d.as_secs();
                            format!("修改时间: {}", format_timestamp(secs))
                        })
                } else {
                    None
                };
                result.push(ExportOption {
                    id: format!("{}_dir_{}", parent_id, name),
                    title: name.to_string(),
                    description,
                    rules: Some(escape_glob_chars(&rule)),
                    show_rules: None,
                    default_checked: true,
                    checked: true,
                    parent: Some(parent_id.to_string()),
                    enabled: true,
                    visible: true,
                });
            }
        }
    }

    result
}

/// 检查实例目录下是否存在指定文件或目录之一
fn has_file_or_dir(instance_dir: &Path, names: &[&str]) -> bool {
    names.iter().any(|n| instance_dir.join(n).exists())
}

/// 检查是否有 Licence 文件（LICEN* 通配）
fn has_licence_file(instance_dir: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(instance_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            if let Some(name) = file_name.to_str() {
                if name.to_uppercase().starts_with("LICEN") {
                    return true;
                }
            }
        }
    }
    false
}

/// 转义 glob 特殊字符（选项规则中的文件名可能含 `[` `]` 等）
fn escape_glob_chars(s: &str) -> String {
    // 用 `[x]` 包裹特殊字符进行转义
    s.replace('[', "[[]").replace(']', "[]]").replace('?', "[?]")
}

/// 简单时间戳格式化（yyyy/MM/dd HH:mm）
fn format_timestamp(secs: u64) -> String {
    // 直接用 SystemTime 转 chrono Local 时间格式化
    let time = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs);
    let dt: chrono::DateTime<chrono::Local> = time.into();
    dt.format("%Y/%m/%d %H:%M").to_string()
}
