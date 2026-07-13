//! 版本 Setup 模块
//!
//! 管理每个版本的 setup.ini 文件，记录版本元数据（加载器类型、版本号等）。
//! 参考 PCL2 的 setup.ini 机制。

use super::state::VersionType;
use std::path::{Path, PathBuf};

/// 版本 Setup 信息
#[derive(Debug, Clone)]
pub struct VersionSetup {
    /// 原始 MC 版本号（如 1.20.1）
    pub original_version: String,
    /// 版本类型
    pub version_type: VersionType,
    /// Forge 版本号（如有）
    pub forge_version: Option<String>,
    /// NeoForge 版本号（如有）
    pub neoforge_version: Option<String>,
    /// Fabric Loader 版本号（如有）
    pub fabric_version: Option<String>,
    /// Quilt Loader 版本号（如有）
    pub quilt_version: Option<String>,
    /// OptiFine 版本号（如有）
    pub optifine_version: Option<String>,
    /// LiteLoader 版本号（如有）
    pub liteloader_version: Option<String>,
    /// 自定义图标文件名（空字符串=自动判断，PCL\Logo.png 等相对路径）
    pub logo: Option<String>,
    /// 自定义描述（空字符串=使用默认描述）
    pub custom_info: Option<String>,
    /// 强制版本分类（0=自动，1=隐藏，2=可安装Mod，3=原版类似，4=垃圾，5=愚人节，6=错误）
    pub display_type: Option<i32>,
    /// 是否收藏
    pub is_star: Option<bool>,
}

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
        }
    }

    /// 获取 setup.ini 文件路径
    pub fn file_path(version_dir: &Path) -> PathBuf {
        version_dir.join("setup.ini")
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

        // 若保留个性化字段，先读取旧 setup.ini 中的个性化值
        let (old_logo, old_info, old_dtype, old_star) = if preserve_personalization {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let ini = parse_ini(&content);
                    (
                        ini.get("Logo").cloned(),
                        ini.get("CustomInfo").cloned(),
                        ini.get("DisplayType").and_then(|s| s.parse::<i32>().ok()),
                        ini.get("IsStar").map(|s| s.eq_ignore_ascii_case("true")),
                    )
                }
                Err(_) => (None, None, None, None),
            }
        } else {
            (None, None, None, None)
        };

        let logo = if preserve_personalization {
            old_logo.or_else(|| setup.logo.clone())
        } else {
            setup.logo.clone()
        };
        let custom_info = if preserve_personalization {
            old_info.or_else(|| setup.custom_info.clone())
        } else {
            setup.custom_info.clone()
        };
        let display_type = if preserve_personalization {
            old_dtype.or(setup.display_type)
        } else {
            setup.display_type
        };
        let is_star = if preserve_personalization {
            old_star.or(setup.is_star)
        } else {
            setup.is_star
        };

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

        // 个性化字段（空值也写入，保持一致）
        content.push_str(&format!("Logo={}\n", logo.unwrap_or_default()));
        content.push_str(&format!("CustomInfo={}\n", custom_info.unwrap_or_default()));
        if let Some(dt) = display_type {
            content.push_str(&format!("DisplayType={}\n", dt));
        } else {
            content.push_str("DisplayType=0\n");
        }
        content.push_str(&format!(
            "IsStar={}\n",
            is_star.unwrap_or(false)
        ));

        std::fs::write(&path, content)
    }

    /// 加载或从 JSON 推断（若 setup.ini 不存在则从版本 JSON 推断并保存）
    pub fn load_or_create(version_dir: &Path, version_id: &str) -> Self {
        if let Ok(Some(setup)) = Self::load(version_dir) {
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
            });
        let _ = setup.save(version_dir);
        setup
    }

    /// 更新单个个性化字段（不修改其他字段）
    pub fn update_personalization(
        version_dir: &Path,
        logo: Option<&str>,
        custom_info: Option<&str>,
        display_type: Option<i32>,
        is_star: Option<bool>,
    ) -> std::io::Result<()> {
        let path = Self::file_path(version_dir);
        let mut setup = if path.exists() {
            Self::load(version_dir)?.unwrap_or_else(|| Self {
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
            })
        } else {
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
            }
        };

        if let Some(v) = logo {
            setup.logo = Some(v.to_string());
        }
        if let Some(v) = custom_info {
            setup.custom_info = Some(v.to_string());
        }
        if let Some(v) = display_type {
            setup.display_type = Some(v);
        }
        if let Some(v) = is_star {
            setup.is_star = Some(v);
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
        })
    }
}

/// 从 Maven 坐标提取版本号
fn extract_maven_version(name: &str, prefix: &str) -> Option<String> {
    if name.starts_with(prefix) {
        Some(name[prefix.len()..].to_string())
    } else {
        None
    }
}

/// 简单的 INI 解析器
fn parse_ini(content: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('[')
            || line.starts_with('#')
            || line.starts_with(';')
        {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
