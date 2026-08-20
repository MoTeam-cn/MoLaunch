//! Mod 元数据读取编排：MetaBuilder 累积合并 → 版本号 fallback 与有效性校验 → 译名查询
//! 4 个来源读取函数（merge_*）位于 `sources`。

use super::super::types::ModMeta;
use super::sources::{
    merge_fabric_mod_json, merge_fml_cache_annotation, merge_mcmod_info, merge_mods_toml,
    read_manifest_version,
};
use crate::minecraft::community::types::ModMetadata;

/// 从 jar 文件内读取 mod 元数据：译名、描述、版本号、slug
///
/// 按顺序尝试 4 个来源，累积合并，已有有效值不覆盖。
pub(crate) fn read_mod_metadata(path: &std::path::Path) -> ModMetadata {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return ModMetadata::default(),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return ModMetadata::default(),
    };

    // 累积合并模式：按顺序尝试 4 个来源，已有有效值不覆盖
    let mut builder = MetaBuilder::new();
    merge_mcmod_info(&mut archive, &mut builder);
    merge_fabric_mod_json(&mut archive, &mut builder);
    merge_mods_toml(&mut archive, &mut builder);
    merge_fml_cache_annotation(&mut archive, &mut builder);

    let mut meta = builder.build();

    // 将 Version 代号转换为 META-INF 中的版本
    if meta.version == "version" {
        if let Some(manifest_ver) = read_manifest_version(&mut archive) {
            meta.version = manifest_ver;
        } else {
            meta.version = String::new();
        }
    }

    // 版本号有效性校验：
    // 版本号必须包含 "." 或 "-"，否则视为无效
    if !meta.version.is_empty() && !meta.version.contains('.') && !meta.version.contains('-') {
        meta.version = String::new();
    }

    finalize_metadata(meta, path)
}

/// 中间结构，封装"不覆盖"合并逻辑
///
/// 字段设置规则：
/// - Display: 第一个非空、非占位符的值优先（后续不覆盖）
/// - Description: 第一个长度>2的值优先（后续不覆盖）
/// - Version: 第一个有效版本号优先（后续不覆盖），占位符标记为 "version"
/// - Dependencies: 累积合并，去重保存所有来源声明的 mod_id
pub(super) struct MetaBuilder {
    slug: Option<String>,
    description: Option<String>,
    /// None = 未设置, Some("version") = 占位符标记, Some(其他) = 实际版本号
    version: Option<String>,
    /// 依赖的 mod_id 列表（累积合并，去重）
    dependencies: Vec<String>,
}

impl MetaBuilder {
    pub(super) fn new() -> Self {
        Self {
            slug: None,
            description: None,
            version: None,
            dependencies: Vec::new(),
        }
    }

    /// 设置 slug（只在未设置时赋值）
    pub(super) fn set_slug(&mut self, value: Option<String>) {
        if self.slug.is_some() {
            return;
        }
        if let Some(v) = value {
            let v = v.trim().to_lowercase();
            if !v.is_empty() {
                self.slug = Some(v);
            }
        }
    }

    /// 设置 description（只在未设置且长度>2时赋值）
    pub(super) fn set_description(&mut self, value: String) {
        if self.description.is_some() {
            return;
        }
        let v = value.trim_matches('\n').trim().to_string();
        if v.chars().count() > 2 {
            self.description = Some(v);
        }
    }

    /// 设置 version（已有有效版本不覆盖，占位符标记为 "version"）
    pub(super) fn set_version(&mut self, value: String) {
        if let Some(ref v) = self.version {
            if is_valid_version(v) {
                return;
            }
        }
        if value.to_lowercase().contains("version") {
            self.version = Some("version".to_string());
        } else {
            self.version = Some(value);
        }
    }

    /// 追加依赖的 mod_id 列表（去重，空值跳过）
    ///
    /// 多个来源（fabric.mod.json / mods.toml）可能各自声明依赖，
    /// 全部累积到 `dependencies` 字段，由后续依赖检测统一消费。
    pub(super) fn add_dependencies(&mut self, ids: impl IntoIterator<Item = String>) {
        for id in ids {
            let v = id.trim().to_lowercase();
            if v.is_empty() {
                continue;
            }
            if !self.dependencies.iter().any(|d| d == &v) {
                self.dependencies.push(v);
            }
        }
    }

    /// 转换为 ModMeta
    pub(super) fn build(self) -> ModMeta {
        ModMeta {
            slug: self.slug,
            description: self.description.unwrap_or_default(),
            version: self.version.unwrap_or_default(),
            dependencies: self.dependencies,
        }
    }
}

/// 判断是否为有效版本号（只含数字、点、减号，对应正则 `[0-9.\-]+`）
fn is_valid_version(v: &str) -> bool {
    !v.is_empty()
        && v.chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
}

/// 把中间结构 ModMeta 转换为最终 ModMetadata（查译名 + 版本号 fallback）
fn finalize_metadata(meta: ModMeta, path: &std::path::Path) -> ModMetadata {
    let slug = meta.slug.clone().unwrap_or_default();
    let translated = meta
        .slug
        .as_deref()
        .and_then(lookup_translated)
        .unwrap_or_default();
    // 版本号 fallback：JAR 元数据为空时从文件名提取
    let version = if meta.version.is_empty() {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|name| {
                crate::minecraft::community::version_extract::extract_version_from_name(name)
            })
            .unwrap_or_default()
    } else {
        meta.version
    };
    ModMetadata {
        slug,
        description: meta.description,
        version,
        translated_name: translated,
        dependencies: meta.dependencies,
    }
}

/// 查询 mcmod 译名（先查 CurseForge slug，再查 Modrinth slug）
fn lookup_translated(slug: &str) -> Option<String> {
    let slug = slug.trim().to_lowercase();
    if let Some(name) = crate::minecraft::community::mcmod::lookup_cf(&slug) {
        return Some(name.to_string());
    }
    if let Some(name) = crate::minecraft::community::mcmod::lookup_mr(&slug) {
        return Some(name.to_string());
    }
    None
}
