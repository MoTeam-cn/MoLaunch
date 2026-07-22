//! VersionSetup 保存逻辑
//!
//! - `save`              保留个性化字段的保存
//! - `save_full`         覆盖所有字段的完整保存
//! - `save_with_options` 保存实现（保留策略由参数控制）
//! - `ensure_complete`   按模板补全缺失字段

use std::path::Path;

use super::helpers::parse_ini;
use super::types::VersionSetup;

impl VersionSetup {
    /// 保存到 setup.ini（保留已有个性化字段，仅更新基础信息）
    pub fn save(&self, version_dir: &Path) -> std::io::Result<()> {
        Self::save_with_options(version_dir, self, true)
    }

    /// 完整保存（覆盖所有字段，包括个性化字段）
    pub fn save_full(&self, version_dir: &Path) -> std::io::Result<()> {
        Self::save_with_options(version_dir, self, false)
    }

    /// 保存实现：若 preserve_personalization=true，保留已存在的个性化字段
    fn save_with_options(
        version_dir: &Path,
        setup: &VersionSetup,
        preserve_personalization: bool,
    ) -> std::io::Result<()> {
        let path = Self::file_path(version_dir);

        // 读取旧 setup.ini（用于保留个性化字段）
        let old_ini: std::collections::HashMap<String, String> = if preserve_personalization {
            std::fs::read_to_string(&path)
                .map(|c| parse_ini(&c))
                .unwrap_or_default()
        } else {
            Default::default()
        };

        // 保留策略：preserve 时 old.or(new)，否则直接用 new
        let pick_str = |key: &str, new: &Option<String>| -> String {
            if preserve_personalization {
                old_ini
                    .get(key)
                    .cloned()
                    .filter(|s| !s.is_empty())
                    .or_else(|| new.clone())
                    .unwrap_or_default()
            } else {
                new.clone().unwrap_or_default()
            }
        };
        let pick_i32 = |key: &str, new: Option<i32>| -> Option<i32> {
            if preserve_personalization {
                old_ini.get(key).and_then(|s| s.parse::<i32>().ok()).or(new)
            } else {
                new
            }
        };
        let pick_bool = |key: &str, new: Option<bool>| -> Option<bool> {
            if preserve_personalization {
                old_ini
                    .get(key)
                    .map(|s| s.eq_ignore_ascii_case("true"))
                    .or(new)
            } else {
                new
            }
        };
        let pick_u32 = |key: &str, new: Option<u32>| -> Option<u32> {
            if preserve_personalization {
                old_ini.get(key).and_then(|s| s.parse::<u32>().ok()).or(new)
            } else {
                new
            }
        };

        let logo = pick_str("Logo", &setup.display.logo);
        let custom_info = pick_str("CustomInfo", &setup.display.custom_info);
        let display_type = pick_i32("DisplayType", setup.display.display_type);
        let is_star = pick_bool("IsStar", setup.display.is_star);
        let indie_type = pick_i32("IndieType", setup.display.indie_type);
        let window_title = pick_str("WindowTitle", &setup.display.window_title);
        let server_enter = pick_str("ServerEnter", &setup.display.server_enter);
        let advance_jvm_args = pick_str("AdvanceJvmArgs", &setup.advanced.jvm_args);
        let advance_game_args = pick_str("AdvanceGameArgs", &setup.advanced.game_args);
        let advance_run_cmd = pick_str("AdvanceRunCmd", &setup.advanced.run_cmd);
        let java_path = pick_str("JavaPath", &setup.java.java_path);
        let java_mode = pick_str("JavaMode", &setup.java.java_mode);
        let java_version_min = pick_u32("JavaVersionMin", setup.java.java_version_min);
        let java_version_max = pick_u32("JavaVersionMax", setup.java.java_version_max);
        let memory_mode = pick_str("MemoryMode", &setup.java.memory_mode);
        let min_memory = pick_u32("MinMemory", setup.java.min_memory);
        let max_memory = pick_u32("MaxMemory", setup.java.max_memory);
        let advance_disable_mod_update =
            pick_bool("AdvanceDisableModUpdate", setup.advanced.disable_mod_update);
        let advance_ignore_java_warning = pick_bool(
            "AdvanceIgnoreJavaWarning",
            setup.advanced.ignore_java_warning,
        );
        let advance_disable_assets_verify = pick_bool(
            "AdvanceDisableAssetsVerify",
            setup.advanced.disable_assets_verify,
        );
        let advance_disable_jlw = pick_bool("AdvanceDisableJLW", setup.advanced.disable_jlw);
        let advance_disable_lua = pick_bool("AdvanceDisableLUA", setup.advanced.disable_lua);

        let mut content = String::new();
        content.push_str("[info]\n");
        content.push_str(&format!("OriginalVersion={}\n", setup.loader.original_version));
        content.push_str(&format!("Type={}\n", setup.loader.version_type.as_str()));

        if let Some(ref v) = setup.loader.forge_version {
            content.push_str(&format!("ForgeVersion={}\n", v));
        }
        if let Some(ref v) = setup.loader.neoforge_version {
            content.push_str(&format!("NeoForgeVersion={}\n", v));
        }
        if let Some(ref v) = setup.loader.fabric_version {
            content.push_str(&format!("FabricVersion={}\n", v));
        }
        if let Some(ref v) = setup.loader.quilt_version {
            content.push_str(&format!("QuiltVersion={}\n", v));
        }
        if let Some(ref v) = setup.loader.optifine_version {
            content.push_str(&format!("OptiFineVersion={}\n", v));
        }
        if let Some(ref v) = setup.loader.liteloader_version {
            content.push_str(&format!("LiteLoaderVersion={}\n", v));
        }

        // 个性化字段
        content.push_str(&format!("Logo={}\n", logo));
        content.push_str(&format!("CustomInfo={}\n", custom_info));
        content.push_str(&format!("DisplayType={}\n", display_type.unwrap_or(0)));
        content.push_str(&format!("IsStar={}\n", is_star.unwrap_or(false)));
        if let Some(it) = indie_type {
            content.push_str(&format!("IndieType={}\n", it));
        }
        // 版本功能设置（空值也写入，便于人工编辑）
        content.push_str(&format!("WindowTitle={}\n", window_title));
        content.push_str(&format!("ServerEnter={}\n", server_enter));
        content.push_str(&format!("AdvanceJvmArgs={}\n", advance_jvm_args));
        content.push_str(&format!("AdvanceGameArgs={}\n", advance_game_args));
        content.push_str(&format!("AdvanceRunCmd={}\n", advance_run_cmd));
        content.push_str(&format!(
            "AdvanceDisableModUpdate={}\n",
            advance_disable_mod_update.unwrap_or(false)
        ));
        content.push_str(&format!(
            "AdvanceIgnoreJavaWarning={}\n",
            advance_ignore_java_warning.unwrap_or(false)
        ));
        content.push_str(&format!(
            "AdvanceDisableAssetsVerify={}\n",
            advance_disable_assets_verify.unwrap_or(false)
        ));
        content.push_str(&format!(
            "AdvanceDisableJLW={}\n",
            advance_disable_jlw.unwrap_or(false)
        ));
        content.push_str(&format!(
            "AdvanceDisableLUA={}\n",
            advance_disable_lua.unwrap_or(false)
        ));
        content.push_str(&format!("JavaPath={}\n", java_path));
        content.push_str(&format!("JavaMode={}\n", java_mode));
        if let Some(v) = java_version_min {
            content.push_str(&format!("JavaVersionMin={}\n", v));
        }
        if let Some(v) = java_version_max {
            content.push_str(&format!("JavaVersionMax={}\n", v));
        }

        // 内存设置独立段
        content.push_str("\n[Memory]\n");
        content.push_str(&format!("MemoryMode={}\n", memory_mode));
        if let Some(mm) = min_memory {
            content.push_str(&format!("MinMemory={}\n", mm));
        }
        if let Some(mm) = max_memory {
            content.push_str(&format!("MaxMemory={}\n", mm));
        }

        // 原子写入：先写 .tmp 再 rename，避免崩溃导致 setup.ini 半写状态
        let tmp = path.with_extension("ini.tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// 按模板（resources/defaults/setup.ini）比对补全缺失字段
    /// 返回 true 表示有补全修改，false 表示无需修改
    /// 段感知：[info] 和 [Memory] 段分别比对，只补缺失的 key，不覆盖已有值
    pub fn ensure_complete(version_dir: &Path) -> std::io::Result<bool> {
        let path = Self::file_path(version_dir);
        if !path.exists() {
            return Ok(false);
        }

        // 读取模板
        let template_content = crate::resources::read_resource("defaults/setup.ini")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let template = crate::storage::ini::IniFile::parse(&template_content);

        // 读取当前 setup.ini
        let current_content = std::fs::read_to_string(&path)?;
        let mut current = crate::storage::ini::IniFile::parse(&current_content);

        // 逐段比对，补全缺失的 key
        let modified = current.merge_missing_from(&template);

        if modified {
            // 原子写入：先写 .tmp 再 rename
            let tmp = path.with_extension("ini.tmp");
            std::fs::write(&tmp, current.to_string())?;
            std::fs::rename(&tmp, &path)?;
        }

        Ok(modified)
    }
}
