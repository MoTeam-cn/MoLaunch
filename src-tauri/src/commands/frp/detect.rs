//! 拖拽包类型检测：根据目录或 ZIP 内容特征区分 Frp 厂商包、Minecraft 整合包与未知类型。

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::log_info;

/// 拖拽包类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
    /// frp 厂商包（manifest.json 含 frp 特征字段）
    FrpProvider,
    /// Minecraft 整合包
    Modpack,
    /// 无法识别
    Unknown,
}

/// 检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectPackageResult {
    #[serde(rename = "type")]
    pub package_type: PackageType,
    /// frp 厂商包：manifest 中的 id（用于前端提示"更新厂商 X"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    /// frp 厂商包：manifest 中的 name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_name: Option<String>,
}

/// 检测拖拽包类型
///
/// 支持 .zip（读内容特征）与目录（读 manifest.json）。返回分类结果，
/// 不修改任何文件。失败（无法读取/解析）时返回 Unknown 而非报错，
/// 方便拖拽流程降级为通用提示。
pub fn detect_package_type(path: &str) -> Result<DetectPackageResult, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("文件不存在: {}", path));
    }

    let result = if p.is_dir() {
        detect_from_dir(p)
    } else {
        detect_from_zip(p)
    };

    log_info!(
        "[Frp] 包类型检测: {:?} (path={})",
        result.package_type,
        path
    );
    Ok(result)
}

/// 目录检测：读根 manifest.json
fn detect_from_dir(dir: &Path) -> DetectPackageResult {
    let manifest_path = dir.join("manifest.json");
    if !manifest_path.exists() {
        return DetectPackageResult {
            package_type: PackageType::Unknown,
            provider_id: None,
            provider_name: None,
        };
    }
    let content = std::fs::read_to_string(&manifest_path).unwrap_or_default();
    classify_manifest(&content)
}

/// ZIP 检测：扫描条目定位 manifest.json（根或一级子目录），读取内容分类
fn detect_from_zip(zip_path: &Path) -> DetectPackageResult {
    let file = match std::fs::File::open(zip_path) {
        Ok(f) => f,
        Err(_) => {
            return DetectPackageResult {
                package_type: PackageType::Unknown,
                provider_id: None,
                provider_name: None,
            }
        }
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => {
            return DetectPackageResult {
                package_type: PackageType::Unknown,
                provider_id: None,
                provider_name: None,
            }
        }
    };

    // 定位候选 manifest.json：根级 或 一级子目录（形如 "sub/manifest.json"）
    let mut candidate: Option<(usize, String)> = None;
    for i in 0..archive.len() {
        let name = archive
            .by_index(i)
            .map(|e| e.name().to_string())
            .unwrap_or_default();
        let normalized = name.replace('\\', "/");
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        let is_manifest = parts.last().map(|s| *s == "manifest.json").unwrap_or(false);
        if !is_manifest {
            continue;
        }
        // 优先根级，其次一级子目录；越浅越优先
        let depth = parts.len();
        let better = match &candidate {
            Some((_, n)) => {
                let old_depth = n.split('/').count();
                depth < old_depth
            }
            None => true,
        };
        if better {
            candidate = Some((i, normalized));
        }
    }

    let Some((idx, _)) = candidate else {
        // 没有 manifest.json：若含 modrinth.index.json / mmc-pack.json 等仍属整合包，
        // 此处交给整合包检测逻辑，frp 检测只认 manifest.json
        return DetectPackageResult {
            package_type: PackageType::Unknown,
            provider_id: None,
            provider_name: None,
        };
    };

    let mut entry = match archive.by_index(idx) {
        Ok(e) => e,
        Err(_) => {
            return DetectPackageResult {
                package_type: PackageType::Unknown,
                provider_id: None,
                provider_name: None,
            }
        }
    };
    let mut content = String::new();
    if std::io::Read::read_to_string(&mut entry, &mut content).is_err() {
        return DetectPackageResult {
            package_type: PackageType::Unknown,
            provider_id: None,
            provider_name: None,
        };
    }
    classify_manifest(&content)
}

/// 依据 manifest.json 内容分类
///
/// frp 厂商包特征：`id` + `binary`（或 `api`）同时存在。
/// 整合包特征：`addons`（MCBBS）或 `files` + `minecraft`（CurseForge）——
/// 这类 manifest 不含 binary/api，不会误判为 frp 厂商包。
fn classify_manifest(content: &str) -> DetectPackageResult {
    let value: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => {
            return DetectPackageResult {
                package_type: PackageType::Unknown,
                provider_id: None,
                provider_name: None,
            }
        }
    };

    let has_id = value.get("id").is_some();
    let has_binary = value.get("binary").is_some();
    let has_api = value.get("api").is_some();
    let has_addons = value.get("addons").is_some();
    let has_files = value.get("files").is_some();
    let has_minecraft = value.get("minecraft").is_some();

    // 整合包特征优先（MCBBS/CurseForge 都可能有 id，但它们的 id 是数字/内部标识，
    // 且不含 binary/api 字段）
    if has_addons || (has_files && has_minecraft) {
        return DetectPackageResult {
            package_type: PackageType::Modpack,
            provider_id: None,
            provider_name: None,
        };
    }

    if has_id && (has_binary || has_api) {
        let provider_id = value
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let provider_name = value
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        return DetectPackageResult {
            package_type: PackageType::FrpProvider,
            provider_id,
            provider_name,
        };
    }

    DetectPackageResult {
        package_type: PackageType::Unknown,
        provider_id: None,
        provider_name: None,
    }
}

#[cfg(test)]
#[path = "detect_tests.rs"]
mod tests;
