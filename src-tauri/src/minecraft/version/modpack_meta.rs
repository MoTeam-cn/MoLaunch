//! 整合包元数据本地持久化（`versions/{id}/modpack.meta.json`）
//!
//! 安装时写入，创建联机房间时读取上报。与 `ModpackMeta` 字段一致，不存储 `download_url`（安全考虑）。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::minecraft::online::signaling::ModpackMeta;

/// 本地整合包元数据（`modpack.meta.json` 文件格式）
///
/// 字段与 `ModpackMeta` 一致，额外含 `installed_at` 本地记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackMetaFile {
    /// 来源平台（`curseforge` / `modrinth`）
    pub source: String,
    /// CF project id 或 MR project id
    pub project_id: String,
    /// CF file id 或 MR version id
    pub file_id: String,
    /// 整合包对应的 MC 版本（如 `1.12.2`）
    pub mc_version: String,
    /// 整合包自身版本号（如 `2.9.3`）
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub modpack_version: Option<String>,
    /// 整合包名称
    pub name: String,
    /// 加载器类型（`forge` / `fabric` / `neoforge` / `quilt`）
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub loader: Option<String>,
    /// 加载器版本号
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub loader_version: Option<String>,
    /// 整合包文件大小（字节）
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub file_size: Option<u64>,
    /// mods 文件数
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub file_count: Option<u32>,
    /// manifest.json SHA-256
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub manifest_hash: Option<String>,
    /// 安装时间（Unix 秒，仅本地记录，不上报）
    #[serde(default)]
    pub installed_at: u64,
}

impl ModpackMetaFile {
    /// `modpack.meta.json` 文件名
    const FILE_NAME: &'static str = "modpack.meta.json";

    /// 返回 `versions/{id}/modpack.meta.json` 的完整路径
    pub fn file_path(version_dir: &Path) -> PathBuf {
        version_dir.join(Self::FILE_NAME)
    }

    /// 从 `versions/{id}/modpack.meta.json` 加载元数据
    ///
    /// 文件不存在时返回 `Ok(None)`；解析失败返回错误。
    pub fn load(version_dir: &Path) -> std::io::Result<Option<Self>> {
        let path = Self::file_path(version_dir);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let meta: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some(meta))
    }

    /// 原子写入 `versions/{id}/modpack.meta.json`（tmp → rename）
    ///
    /// 调用方需确保 `version_dir` 已存在。
    pub fn save(&self, version_dir: &Path) -> std::io::Result<()> {
        let path = Self::file_path(version_dir);
        let tmp_path = path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&tmp_path, content)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    /// 转换为上报用 `ModpackMeta`（丢弃 `installed_at` 本地字段）
    pub fn to_signaling_meta(&self) -> ModpackMeta {
        ModpackMeta {
            source: self.source.clone(),
            project_id: self.project_id.clone(),
            file_id: self.file_id.clone(),
            mc_version: self.mc_version.clone(),
            modpack_version: self.modpack_version.clone(),
            name: self.name.clone(),
            loader: self.loader.clone(),
            loader_version: self.loader_version.clone(),
            file_size: self.file_size,
            file_count: self.file_count,
            manifest_hash: self.manifest_hash.clone(),
        }
    }
}

/// 计算 manifest.json 内容的 SHA-256 哈希（十六进制字符串）
///
/// 用于加入方校验本地是否已安装同款整合包。
pub fn compute_manifest_hash(content: &[u8]) -> String {
    crate::utils::hash::sha256_hex(content)
}
