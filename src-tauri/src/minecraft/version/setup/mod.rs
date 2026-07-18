//! 版本 Setup 模块
//!
//! 管理每个版本的 setup.ini 文件，记录版本元数据（加载器类型、版本号等）。
//! 参考 PCL2 的 setup.ini 机制。
//!
//! 按关注点拆分为 3 个子模块：
//! - `types`     PersonalizationUpdate + VersionSetup 结构体定义
//! - `helpers`   parse_ini / extract_maven_version / read_setup_version_and_loader / read_mc_version_from_json / detect_version_and_loader
//! - `mod.rs`    impl VersionSetup（构造 + save/load + ensure_complete + load_or_create + update_personalization + from_version_json）+ tests

mod helpers;
mod types;

pub use helpers::{detect_version_and_loader, read_mc_version_from_json, read_setup_version_and_loader};
pub use types::{PersonalizationUpdate, VersionSetup};

use super::state::VersionType;
use std::path::{Path, PathBuf};

use helpers::{extract_maven_version, parse_ini};

impl VersionSetup {
    /// 创建新的 Setup（安装时调用）
    pub fn new(
        original_version: &str,
        version_type: VersionType,
        forge: Option<&str>,
        neoforge: Option<&str>,
        fabric: Option<&str>,
        quilt: Option<&str>,
        optifine: Option<&str>,
        liteloader: Option<&str>,
    ) -> Self {
        Self {
            original_version: original_version.to_string(),
            version_type,
            forge_version: forge.map(|s| s.to_string()),
            neoforge_version: neoforge.map(|s| s.to_string()),
            fabric_version: fabric.map(|s| s.to_string()),
            quilt_version: quilt.map(|s| s.to_string()),
            optifine_version: optifine.map(|s| s.to_string()),
            liteloader_version: liteloader.map(|s| s.to_string()),
            logo: None,
            custom_info: None,
            display_type: None,
            is_star: None,
            indie_type: None,
            window_title: None,
            server_enter: None,
            advance_jvm_args: None,
            advance_game_args: None,
            advance_run_cmd: None,
            java_path: None,
            java_mode: None,
            java_version_min: None,
            java_version_max: None,
            memory_mode: None,
            min_memory: None,
            max_memory: None,
            advance_disable_mod_update: None,
            advance_ignore_java_warning: None,
            advance_disable_assets_verify: None,
            advance_disable_jlw: None,
            advance_disable_lua: None,
        }
    }

    /// 获取 setup.ini 文件路径
    pub fn file_path(version_dir: &Path) -> PathBuf {
        version_dir.join("setup.ini")
    }

    /// 全空默认 Setup（用于 setup.ini 不存在时的兜底）
    pub fn empty() -> Self {
        Self {
            original_version: String::new(),
            version_type: VersionType::Unknown,
            forge_version: None,
            neoforge_version: None,
            fabric_version: None,
            quilt_version: None,
            optifine_version: None,
            liteloader_version: None,
            logo: None,
            custom_info: None,
            display_type: None,
            is_star: None,
            indie_type: None,
            window_title: None,
            server_enter: None,
            advance_jvm_args: None,
            advance_game_args: None,
            advance_run_cmd: None,
            java_path: None,
            java_mode: None,
            java_version_min: None,
            java_version_max: None,
            memory_mode: None,
            min_memory: None,
            max_memory: None,
            advance_disable_mod_update: None,
            advance_ignore_java_warning: None,
            advance_disable_assets_verify: None,
            advance_disable_jlw: None,
            advance_disable_lua: None,
        }
    }

    /// 检查 setup.ini 是否存在
    pub fn exists(version_dir: &Path) -> bool {
        Self::file_path(version_dir).exists()
    }

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
                old_ini
                    .get(key)
                    .and_then(|s| s.parse::<i32>().ok())
                    .or(new)
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
                old_ini
                    .get(key)
                    .and_then(|s| s.parse::<u32>().ok())
                    .or(new)
            } else {
                new
            }
        };

        let logo = pick_str("Logo", &setup.logo);
        let custom_info = pick_str("CustomInfo", &setup.custom_info);
        let display_type = pick_i32("DisplayType", setup.display_type);
        let is_star = pick_bool("IsStar", setup.is_star);
        let indie_type = pick_i32("IndieType", setup.indie_type);
        let window_title = pick_str("WindowTitle", &setup.window_title);
        let server_enter = pick_str("ServerEnter", &setup.server_enter);
        let advance_jvm_args = pick_str("AdvanceJvmArgs", &setup.advance_jvm_args);
        let advance_game_args = pick_str("AdvanceGameArgs", &setup.advance_game_args);
        let advance_run_cmd = pick_str("AdvanceRunCmd", &setup.advance_run_cmd);
        let java_path = pick_str("JavaPath", &setup.java_path);
        let java_mode = pick_str("JavaMode", &setup.java_mode);
        let java_version_min = pick_u32("JavaVersionMin", setup.java_version_min);
        let java_version_max = pick_u32("JavaVersionMax", setup.java_version_max);
        let memory_mode = pick_str("MemoryMode", &setup.memory_mode);
        let min_memory = pick_u32("MinMemory", setup.min_memory);
        let max_memory = pick_u32("MaxMemory", setup.max_memory);
        let advance_disable_mod_update = pick_bool("AdvanceDisableModUpdate", setup.advance_disable_mod_update);
        let advance_ignore_java_warning = pick_bool("AdvanceIgnoreJavaWarning", setup.advance_ignore_java_warning);
        let advance_disable_assets_verify = pick_bool("AdvanceDisableAssetsVerify", setup.advance_disable_assets_verify);
        let advance_disable_jlw = pick_bool("AdvanceDisableJLW", setup.advance_disable_jlw);
        let advance_disable_lua = pick_bool("AdvanceDisableLUA", setup.advance_disable_lua);

        let mut content = String::new();
        content.push_str("[info]\n");
        content.push_str(&format!("OriginalVersion={}\n", setup.original_version));
        content.push_str(&format!("Type={}\n", setup.version_type.as_str()));

        if let Some(ref v) = setup.forge_version {
            content.push_str(&format!("ForgeVersion={}\n", v));
        }
        if let Some(ref v) = setup.neoforge_version {
            content.push_str(&format!("NeoForgeVersion={}\n", v));
        }
        if let Some(ref v) = setup.fabric_version {
            content.push_str(&format!("FabricVersion={}\n", v));
        }
        if let Some(ref v) = setup.quilt_version {
            content.push_str(&format!("QuiltVersion={}\n", v));
        }
        if let Some(ref v) = setup.optifine_version {
            content.push_str(&format!("OptiFineVersion={}\n", v));
        }
        if let Some(ref v) = setup.liteloader_version {
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
        content.push_str(&format!("AdvanceDisableModUpdate={}\n", advance_disable_mod_update.unwrap_or(false)));
        content.push_str(&format!("AdvanceIgnoreJavaWarning={}\n", advance_ignore_java_warning.unwrap_or(false)));
        content.push_str(&format!("AdvanceDisableAssetsVerify={}\n", advance_disable_assets_verify.unwrap_or(false)));
        content.push_str(&format!("AdvanceDisableJLW={}\n", advance_disable_jlw.unwrap_or(false)));
        content.push_str(&format!("AdvanceDisableLUA={}\n", advance_disable_lua.unwrap_or(false)));
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

    /// 加载或从 JSON 推断（若 setup.ini 不存在则从版本 JSON 推断并保存）
    /// 若 setup.ini 是旧格式（缺失字段），自动按模板补全所有字段
    pub fn load_or_create(version_dir: &Path, version_id: &str) -> Self {
        if let Ok(Some(setup)) = Self::load(version_dir) {
            // 自动补全：按模板比对，缺失字段自动补上
            if Self::ensure_complete(version_dir).unwrap_or(false) {
                // 补全后重新加载，返回完整数据
                if let Ok(Some(refreshed)) = Self::load(version_dir) {
                    return refreshed;
                }
            }
            return setup;
        }
        let setup = Self::from_version_json(version_dir, version_id)
            .unwrap_or_else(|| Self {
                original_version: version_id.to_string(),
                version_type: VersionType::Unknown,
                forge_version: None,
                neoforge_version: None,
                fabric_version: None,
                quilt_version: None,
                optifine_version: None,
                liteloader_version: None,
                logo: None,
                custom_info: None,
                display_type: None,
                is_star: None,
                indie_type: None,
                window_title: None,
                server_enter: None,
                advance_jvm_args: None,
                advance_game_args: None,
                advance_run_cmd: None,
                java_path: None,
                java_mode: None,
                java_version_min: None,
                java_version_max: None,
                memory_mode: None,
                min_memory: None,
                max_memory: None,
                advance_disable_mod_update: None,
                advance_ignore_java_warning: None,
                advance_disable_assets_verify: None,
                advance_disable_jlw: None,
                advance_disable_lua: None,
            });
        let _ = setup.save(version_dir);
        setup
    }

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
            setup.logo = Some(v.clone());
        }
        if let Some(ref v) = update.custom_info {
            setup.custom_info = Some(v.clone());
        }
        if let Some(v) = update.display_type {
            setup.display_type = Some(v);
        }
        if let Some(v) = update.is_star {
            setup.is_star = Some(v);
        }
        if let Some(v) = update.indie_type {
            setup.indie_type = Some(v);
        }
        if let Some(ref v) = update.window_title {
            setup.window_title = Some(v.clone());
        }
        if let Some(ref v) = update.server_enter {
            setup.server_enter = Some(v.clone());
        }
        if let Some(ref v) = update.advance_jvm_args {
            setup.advance_jvm_args = Some(v.clone());
        }
        if let Some(ref v) = update.advance_game_args {
            setup.advance_game_args = Some(v.clone());
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
            setup.advance_run_cmd = Some(v.clone());
        }
        if let Some(ref v) = update.java_path {
            setup.java_path = Some(v.clone());
        }
        if let Some(ref v) = update.java_mode {
            setup.java_mode = Some(v.clone());
        }
        if let Some(v) = update.java_version_min {
            setup.java_version_min = Some(v);
        }
        if let Some(v) = update.java_version_max {
            setup.java_version_max = Some(v);
        }
        if let Some(ref v) = update.memory_mode {
            setup.memory_mode = Some(v.clone());
        }
        if let Some(v) = update.min_memory {
            setup.min_memory = Some(v);
        }
        if let Some(v) = update.max_memory {
            setup.max_memory = Some(v);
        }
        if let Some(v) = update.advance_disable_mod_update {
            setup.advance_disable_mod_update = Some(v);
        }
        if let Some(v) = update.advance_ignore_java_warning {
            setup.advance_ignore_java_warning = Some(v);
        }
        if let Some(v) = update.advance_disable_assets_verify {
            setup.advance_disable_assets_verify = Some(v);
        }
        if let Some(v) = update.advance_disable_jlw {
            setup.advance_disable_jlw = Some(v);
        }
        if let Some(v) = update.advance_disable_lua {
            setup.advance_disable_lua = Some(v);
        }

        setup.save_full(version_dir)
    }

    /// 从 setup.ini 加载
    pub fn load(version_dir: &Path) -> std::io::Result<Option<Self>> {
        let path = Self::file_path(version_dir);
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let ini = parse_ini(&content);

        let original_version = ini
            .get("OriginalVersion")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let version_type = ini
            .get("Type")
            .map(|s| VersionType::from_str(s))
            .unwrap_or(VersionType::Unknown);

        Ok(Some(Self {
            original_version,
            version_type,
            forge_version: ini.get("ForgeVersion").cloned(),
            neoforge_version: ini.get("NeoForgeVersion").cloned(),
            fabric_version: ini.get("FabricVersion").cloned(),
            quilt_version: ini.get("QuiltVersion").cloned(),
            optifine_version: ini.get("OptiFineVersion").cloned(),
            liteloader_version: ini.get("LiteLoaderVersion").cloned(),
            logo: ini.get("Logo").cloned(),
            custom_info: ini.get("CustomInfo").cloned(),
            display_type: ini.get("DisplayType").and_then(|s| s.parse::<i32>().ok()),
            is_star: ini.get("IsStar").map(|s| s.eq_ignore_ascii_case("true")),
            indie_type: ini.get("IndieType").and_then(|s| s.parse::<i32>().ok()),
            window_title: ini.get("WindowTitle").cloned(),
            server_enter: ini.get("ServerEnter").cloned(),
            advance_jvm_args: ini.get("AdvanceJvmArgs").cloned(),
            advance_game_args: ini.get("AdvanceGameArgs").cloned(),
            advance_run_cmd: ini.get("AdvanceRunCmd").cloned(),
            java_path: ini.get("JavaPath").cloned(),
            java_mode: ini.get("JavaMode").cloned(),
            java_version_min: ini.get("JavaVersionMin").and_then(|s| s.parse::<u32>().ok()),
            java_version_max: ini.get("JavaVersionMax").and_then(|s| s.parse::<u32>().ok()),
            memory_mode: ini.get("MemoryMode").cloned(),
            min_memory: ini.get("MinMemory").and_then(|s| s.parse::<u32>().ok()),
            max_memory: ini.get("MaxMemory").and_then(|s| s.parse::<u32>().ok()),
            advance_disable_mod_update: ini.get("AdvanceDisableModUpdate").map(|s| s.eq_ignore_ascii_case("true")),
            advance_ignore_java_warning: ini.get("AdvanceIgnoreJavaWarning").map(|s| s.eq_ignore_ascii_case("true")),
            advance_disable_assets_verify: ini.get("AdvanceDisableAssetsVerify").map(|s| s.eq_ignore_ascii_case("true")),
            advance_disable_jlw: ini.get("AdvanceDisableJLW").map(|s| s.eq_ignore_ascii_case("true")),
            advance_disable_lua: ini.get("AdvanceDisableLUA").map(|s| s.eq_ignore_ascii_case("true")),
        }))
    }

    /// 从版本 JSON 文件推断 Setup（兼容旧版本）
    pub fn from_version_json(version_dir: &Path, version_id: &str) -> Option<Self> {
        let json_path = version_dir.join(format!("{}.json", version_id));
        if !json_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&json_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;

        let version_type = super::state::detect_version_type(version_id, &json);
        let original_version = json["inheritsFrom"]
            .as_str()
            .or_else(|| json["id"].as_str())
            .unwrap_or(version_id)
            .to_string();

        // 从 libraries 提取加载器版本
        let mut forge_version = None;
        let mut neoforge_version = None;
        let mut fabric_version = None;
        let mut quilt_version = None;
        let mut optifine_version = None;
        let mut liteloader_version = None;

        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if let Some(ver) = extract_maven_version(name, "net.minecraftforge:forge:") {
                        forge_version = Some(ver);
                    } else if let Some(ver) = extract_maven_version(name, "net.neoforged:neoforge:")
                    {
                        neoforge_version = Some(ver);
                    } else if let Some(ver) =
                        extract_maven_version(name, "net.fabricmc:fabric-loader:")
                    {
                        fabric_version = Some(ver);
                    } else if let Some(ver) =
                        extract_maven_version(name, "org.quiltmc:quilt-loader:")
                    {
                        quilt_version = Some(ver);
                    } else if let Some(ver) = extract_maven_version(name, "optifine:OptiFine:") {
                        optifine_version = Some(ver);
                    } else if let Some(ver) = extract_maven_version(name, "com.mumfrey:liteloader:")
                    {
                        liteloader_version = Some(ver);
                    }
                }
            }
        }

        Some(Self {
            original_version,
            version_type,
            forge_version,
            neoforge_version,
            fabric_version,
            quilt_version,
            optifine_version,
            liteloader_version,
            logo: None,
            custom_info: None,
            display_type: None,
            is_star: None,
            indie_type: None,
            window_title: None,
            server_enter: None,
            advance_jvm_args: None,
            advance_game_args: None,
            advance_run_cmd: None,
            java_path: None,
            java_mode: None,
            java_version_min: None,
            java_version_max: None,
            memory_mode: None,
            min_memory: None,
            max_memory: None,
            advance_disable_mod_update: None,
            advance_ignore_java_warning: None,
            advance_disable_assets_verify: None,
            advance_disable_jlw: None,
            advance_disable_lua: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini() {
        let content = "[info]\nOriginalVersion=1.20.1\nType=forge\nForgeVersion=47.2.0\n";
        let ini = parse_ini(content);
        assert_eq!(ini.get("OriginalVersion").unwrap(), "1.20.1");
        assert_eq!(ini.get("Type").unwrap(), "forge");
        assert_eq!(ini.get("ForgeVersion").unwrap(), "47.2.0");
    }

    #[test]
    fn test_extract_maven_version() {
        assert_eq!(
            extract_maven_version(
                "net.minecraftforge:forge:1.20.1-47.2.0",
                "net.minecraftforge:forge:"
            ),
            Some("1.20.1-47.2.0".to_string())
        );
        assert_eq!(
            extract_maven_version(
                "net.fabricmc:fabric-loader:0.16.0",
                "net.fabricmc:fabric-loader:"
            ),
            Some("0.16.0".to_string())
        );
        assert_eq!(
            extract_maven_version("other:lib:1.0", "net.minecraftforge:forge:"),
            None
        );
    }
}
