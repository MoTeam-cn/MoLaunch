//! VersionSetup 个性化字段更新逻辑
//!
//! - `update_personalization` 仅更新非 None 的字段，其他保持不变

use std::path::Path;

use super::types::{PersonalizationUpdate, VersionSetup};

impl VersionSetup {
    /// 更新个性化字段（仅更新非 None 的字段，其他保持不变）
    pub fn update_personalization(
        version_dir: &Path,
        update: &PersonalizationUpdate,
    ) -> std::io::Result<()> {
        let path = Self::file_path(version_dir);
        let mut setup = if path.exists() {
            Self::load(version_dir)?.unwrap_or_else(|| Self::empty())
        } else {
            Self::empty()
        };

        if let Some(ref v) = update.logo {
            setup.display.logo = Some(v.clone());
        }
        if let Some(ref v) = update.custom_info {
            setup.display.custom_info = Some(v.clone());
        }
        if let Some(v) = update.display_type {
            setup.display.display_type = Some(v);
        }
        if let Some(v) = update.is_star {
            setup.display.is_star = Some(v);
        }
        if let Some(v) = update.indie_type {
            setup.display.indie_type = Some(v);
        }
        if let Some(ref v) = update.window_title {
            setup.display.window_title = Some(v.clone());
        }
        if let Some(ref v) = update.server_enter {
            setup.display.server_enter = Some(v.clone());
        }
        if let Some(ref v) = update.advance_jvm_args {
            setup.advanced.jvm_args = Some(v.clone());
        }
        if let Some(ref v) = update.advance_game_args {
            setup.advanced.game_args = Some(v.clone());
        }
        if let Some(ref v) = update.advance_run_cmd {
            // 安全检测：记录危险字符警告（CWE-78，不阻止保存）
            // 这些字符可能被用于命令注入，恶意整合包可借此实现 RCE
            if v.contains('&')
                || v.contains('|')
                || v.contains('>')
                || v.contains('<')
                || v.contains('`')
                || v.contains("$(")
            {
                crate::log_warn!(
                    "[Setup] advance_run_cmd contains potentially dangerous characters: {:?}",
                    v
                );
            }
            setup.advanced.run_cmd = Some(v.clone());
        }
        if let Some(ref v) = update.java_path {
            setup.java.java_path = Some(v.clone());
        }
        if let Some(ref v) = update.java_mode {
            setup.java.java_mode = Some(v.clone());
        }
        if let Some(v) = update.java_version_min {
            setup.java.java_version_min = Some(v);
        }
        if let Some(v) = update.java_version_max {
            setup.java.java_version_max = Some(v);
        }
        if let Some(ref v) = update.memory_mode {
            setup.java.memory_mode = Some(v.clone());
        }
        if let Some(v) = update.min_memory {
            setup.java.min_memory = Some(v);
        }
        if let Some(v) = update.max_memory {
            setup.java.max_memory = Some(v);
        }
        if let Some(v) = update.advance_disable_mod_update {
            setup.advanced.disable_mod_update = Some(v);
        }
        if let Some(v) = update.advance_ignore_java_warning {
            setup.advanced.ignore_java_warning = Some(v);
        }
        if let Some(v) = update.advance_disable_assets_verify {
            setup.advanced.disable_assets_verify = Some(v);
        }
        if let Some(v) = update.advance_disable_jlw {
            setup.advanced.disable_jlw = Some(v);
        }
        if let Some(v) = update.advance_disable_lua {
            setup.advanced.disable_lua = Some(v);
        }

        setup.save_full(version_dir)
    }
}
