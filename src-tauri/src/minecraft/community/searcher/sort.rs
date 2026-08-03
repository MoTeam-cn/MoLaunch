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
fn score(p: &ResourceProject, has_query: bool, rtype: ResourceType) -> f64 {
    // 下载量权重（log10，10亿时为1.0）
    let dl_mult = get_download_count_mult(p.platform, rtype);
    let dl_score = (p.download_count as f64 * dl_mult).max(1.0).log10() / 9.0;

    if has_query {
        // 有搜索词：下载量权重 + 名称匹配度
        let name_score = 0.5; // 简化处理，实际可计算相似度
        name_score + dl_score
    } else {
        // 无搜索词：按下载量排序
        dl_score
    }
}

/// 平台下载量权重
fn get_download_count_mult(platform: Platform, rtype: ResourceType) -> f64 {
    match (rtype, platform) {
        (ResourceType::Mod, Platform::CurseForge) => 1.0,
        (ResourceType::Mod, Platform::Modrinth) => 5.0,
        (ResourceType::ModPack, Platform::CurseForge) => 1.0,
        (ResourceType::ModPack, Platform::Modrinth) => 5.0,
        (ResourceType::DataPack, Platform::CurseForge) => 10.0,
        (ResourceType::DataPack, Platform::Modrinth) => 1.0,
        (ResourceType::ResourcePack, Platform::CurseForge) => 1.0,
        (ResourceType::ResourcePack, Platform::Modrinth) => 4.0,
        (ResourceType::Shader, Platform::CurseForge) => 1.0,
        (ResourceType::Shader, Platform::Modrinth) => 4.0,
    }
}
