//! 预加载相关数据类型
//!
//! - `PreloadUpdate`：推送给前端的事件 payload
//! - `PreloadModInput`：命令层传入的预加载输入
//! - `HashedMod`：JAR 元数据 + 双平台 hash 的中间结果（子模块间传递）

use serde::Serialize;

use crate::commands::version::mods::ModMetadata;
use crate::minecraft::community::types::ResourceProject;

/// 单条预加载结果（推送给前端的事件 payload）
///
/// 前端按 `file_name` 匹配对应 mod，更新所有非 null 字段。
/// 元数据字段和 project 可能分两次 emit（元数据先、project 后）。
#[derive(Debug, Clone, Serialize)]
pub struct PreloadUpdate {
    /// 本地 mod 文件名（前端按此字段匹配更新对应 mod）
    pub file_name: String,
    /// JAR 内读到的 slug（空字符串表示未读到，前端不更新）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// JAR 内读到的描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JAR 内读到的版本号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// 平台工程的 logo_url 经过 image_cache::get_image_url 处理后的缓存 URL
    ///
    /// 设计参考 PCL2 ResourceProject.ApplyLogoToMyImage + 本项目皮肤/披风 cached_url：
    /// - 命中缓存：返回 `cache-image://{hash}.png`，零网络请求
    /// - 未命中：返回原始远程 URL，后端异步下载到缓存，完成后 emit `image-cached` 事件
    ///
    /// 仅在 online_query 阶段 project 被填充时一起填充。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_logo_url: Option<String>,
    /// mcmod 数据库查到的中文译名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_name: Option<String>,
    /// CF/MR 查到的平台工程（None 表示未查到或尚未查询）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ResourceProject>,
}

/// 预加载输入：每个 mod 的文件名和绝对路径
#[derive(Debug, Clone)]
pub struct PreloadModInput {
    pub file_name: String,
    pub path: String,
}

/// JAR 元数据 + 双平台 hash 的中间结果
///
/// 由 `jar_metadata::read_jar_metadata_and_hash` 产出，
/// 供 `online_query::query_and_merge` 消费。
#[derive(Clone)]
pub(crate) struct HashedMod {
    pub file_name: String,
    pub metadata: ModMetadata,
    pub cf_fingerprint: Option<u32>,
    pub mr_sha1: Option<String>,
}
