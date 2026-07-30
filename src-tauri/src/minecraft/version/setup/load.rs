//! VersionSetup 加载逻辑：load / load_or_create / from_version_json

use std::path::Path;

use super::super::state::VersionType;
use super::helpers::{extract_maven_version, parse_ini};
use super::types::{AdvancedConfig, DisplayConfig, JavaConfig, LoaderInfo, VersionSetup};

impl VersionSetup {
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
            loader: LoaderInfo {
                original_version,
                version_type,
                forge_version: ini.get("ForgeVersion").cloned(),
                neoforge_version: ini.get("NeoForgeVersion").cloned(),
                fabric_version: ini.get("FabricVersion").cloned(),
                quilt_version: ini.get("QuiltVersion").cloned(),
                optifine_version: ini.get("OptiFineVersion").cloned(),
                liteloader_version: ini.get("LiteLoaderVersion").cloned(),
            },
            display: DisplayConfig {
                logo: ini.get("Logo").cloned(),
                custom_info: ini.get("CustomInfo").cloned(),
                display_type: ini.get("DisplayType").and_then(|s| s.parse::<i32>().ok()),
                is_star: ini.get("IsStar").map(|s| s.eq_ignore_ascii_case("true")),
                indie_type: ini.get("IndieType").and_then(|s| s.parse::<i32>().ok()),
                window_title: ini.get("WindowTitle").cloned(),
                server_enter: ini.get("ServerEnter").cloned(),
            },
            advanced: AdvancedConfig {
                jvm_args: ini.get("AdvanceJvmArgs").cloned(),
                game_args: ini.get("AdvanceGameArgs").cloned(),
                run_cmd: ini.get("AdvanceRunCmd").cloned(),
                disable_mod_update: ini
                    .get("AdvanceDisableModUpdate")
                    .map(|s| s.eq_ignore_ascii_case("true")),
                ignore_java_warning: ini
                    .get("AdvanceIgnoreJavaWarning")
                    .map(|s| s.eq_ignore_ascii_case("true")),
                disable_assets_verify: ini
                    .get("AdvanceDisableAssetsVerify")
                    .map(|s| s.eq_ignore_ascii_case("true")),
                disable_jlw: ini
                    .get("AdvanceDisableJLW")
                    .map(|s| s.eq_ignore_ascii_case("true")),
                disable_lua: ini
                    .get("AdvanceDisableLUA")
                    .map(|s| s.eq_ignore_ascii_case("true")),
            },
            java: JavaConfig {
                java_path: ini.get("JavaPath").cloned(),
                java_mode: ini.get("JavaMode").cloned(),
                java_version_min: ini
                    .get("JavaVersionMin")
                    .and_then(|s| s.parse::<u32>().ok()),
                java_version_max: ini
                    .get("JavaVersionMax")
                    .and_then(|s| s.parse::<u32>().ok()),
                memory_mode: ini.get("MemoryMode").cloned(),
                min_memory: ini.get("MinMemory").and_then(|s| s.parse::<u32>().ok()),
                max_memory: ini.get("MaxMemory").and_then(|s| s.parse::<u32>().ok()),
            },
        }))
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
        let setup = Self::from_version_json(version_dir, version_id).unwrap_or_else(|| Self {
            loader: LoaderInfo {
                original_version: version_id.to_string(),
                version_type: VersionType::Unknown,
                forge_version: None,
                neoforge_version: None,
                fabric_version: None,
                quilt_version: None,
                optifine_version: None,
                liteloader_version: None,
            },
            display: DisplayConfig::default(),
            java: JavaConfig::default(),
            advanced: AdvancedConfig::default(),
        });
        let _ = setup.save(version_dir);
        setup
    }

    /// 从版本 JSON 文件推断 Setup（兼容旧版本）
    pub fn from_version_json(version_dir: &Path, version_id: &str) -> Option<Self> {
        let json_path = version_dir.join(format!("{}.json", version_id));
        if !json_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&json_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;

        let version_type = super::super::state::detect_version_type(version_id, &json);
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
            loader: LoaderInfo {
                original_version,
                version_type,
                forge_version,
                neoforge_version,
                fabric_version,
                quilt_version,
                optifine_version,
                liteloader_version,
            },
            display: DisplayConfig::default(),
            java: JavaConfig::default(),
            advanced: AdvancedConfig::default(),
        })
    }
}
