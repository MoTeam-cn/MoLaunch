//! 结果排序与去重
//!
//! 跨平台去重、名称归一化与下载量加权排序。

use super::super::types::{Platform, ResourceProject, ResourceType};

/// 跨平台去重
pub(super) fn dedup(projects: Vec<ResourceProject>) -> Vec<ResourceProject> {
    let mut result: Vec<ResourceProject> = Vec::new();
    for p in projects {
        let mut is_dup = false;
        for existing in &result {
            if is_like(&p, existing) {
                is_dup = true;
                break;
            }
        }
        if !is_dup {
            result.push(p);
        }
    }
    result
}

/// 判断两个工程是否相同（跨平台）
fn is_like(a: &ResourceProject, b: &ResourceProject) -> bool {
    if a.platform == b.platform {
        return false;
    }
    // 提取字母数字部分比较
    let a_name = alnum_only(&a.raw_name);
    let b_name = alnum_only(&b.raw_name);
    if a_name.is_empty() || b_name.is_empty() {
        return false;
    }
    // 名称相似度（完全匹配或包含关系）
    if a_name == b_name || a_name.contains(&b_name) || b_name.contains(&a_name) {
        return true;
    }
    // slug 匹配
    if !a.slug.is_empty() && a.slug == b.slug {
        return true;
    }
    false
}

fn alnum_only(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

/// 排序
pub(super) fn sort_projects(
    mut projects: Vec<ResourceProject>,
    has_query: bool,
    rtype: ResourceType,
) -> Vec<ResourceProject> {
    projects.sort_by(|a, b| {
        let sa = score(a, has_query, rtype);
        let sb = score(b, has_query, rtype);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    projects
}

/// 计算排序分
///
/// 排序分由「归一化下载量」构成：
/// - 下载量先乘以平台/类型加权系数（补偿不同平台间的生态体量差异），再取对数归一化；
/// - 有搜索词时叠加固定的名称匹配加分（当前为简化实现）。
fn score(p: &ResourceProject, has_query: bool, rtype: ResourceType) -> f64 {
    let dl_mult = platform_weight(p.platform, rtype);
    let dl_score = (p.download_count as f64 * dl_mult).max(1.0).log10() / 9.0;

    if has_query {
        let name_score = 0.5;
        name_score + dl_score
    } else {
        dl_score
    }
}

/// 平台/类型加权系数
///
/// 设计依据：不同平台的「下载量」量纲差异较大（例如 Modrinth 以单项目独立计数、
/// CurseForge 聚合多文件下载），直接比较原始数值会系统性偏向某一平台。
/// 此处按项目类型对平台间的量纲差异做粗略补偿，数值为本项目自定，可随运营数据调整。
fn platform_weight(platform: Platform, rtype: ResourceType) -> f64 {
    use ResourceType::*;
    match (rtype, platform) {
        // Mod：Modrinth 下载量以项目维度统计，与 CurseForge 文件维度存在量级差
        (Mod, Platform::CurseForge) => 1.0,
        (Mod, Platform::Modrinth) => 3.0,
        // 整合包：两平台量纲接近，轻微偏向 Modrinth
        (ModPack, Platform::CurseForge) => 2.0,
        (ModPack, Platform::Modrinth) => 3.0,
        // 数据包：CurseForge 数据包下载量远高于 Modrinth
        (DataPack, Platform::CurseForge) => 6.0,
        (DataPack, Platform::Modrinth) => 1.0,
        // 资源包 / 光影：Modrinth 生态更活跃，相应加权
        (ResourcePack, Platform::CurseForge) => 1.0,
        (ResourcePack, Platform::Modrinth) => 3.0,
        (Shader, Platform::CurseForge) => 1.0,
        (Shader, Platform::Modrinth) => 3.0,
    }
}
