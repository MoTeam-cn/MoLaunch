//! 双平台搜索调度器
//!
//! 参考 PCL2 ResourceSearcher.Search
//! 并行调用 CurseForge 和 Modrinth，合并结果、去重、排序

use super::types::{ResourceProject, SearchParams, SearchResult};

/// 每页结果数
pub const PAGE_SIZE: u32 = 40;

/// 单平台搜索超时（秒），防止单个慢平台拖慢整体
const PLATFORM_TIMEOUT_SECS: u64 = 15;

/// 搜索入口
pub async fn search(params: SearchParams) -> Result<SearchResult, String> {
    let source = params.source; // 0=全部, 1=仅CF, 2=仅MR
    let rtype = params.resource_type;
    let game_version = params.game_version.as_deref();
    let category = params.category.as_deref();

    let mut cf_fut = None;
    let mut mr_fut = None;

    // 根据来源筛选决定调用哪些平台
    if source == 0 || source == 1 {
        cf_fut = Some(super::curseforge::search(
            &params.query,
            rtype,
            game_version,
            params.mod_loader,
            category,
            params.page,
        ));
    }
    if source == 0 || source == 2 {
        mr_fut = Some(super::modrinth::search(
            &params.query,
            rtype,
            game_version,
            params.mod_loader,
            category,
            params.page,
        ));
    }

    // 每个平台独立超时，一个慢/失败不阻塞另一个
    let (cf_result, mr_result) = tokio::join!(
        async {
            match cf_fut {
                Some(f) => match tokio::time::timeout(
                    std::time::Duration::from_secs(PLATFORM_TIMEOUT_SECS),
                    f,
                ).await {
                    Ok(Ok(r)) => Some(r),
                    Ok(Err(e)) => {
                        crate::log_warn!("[Community] CF 搜索失败: {}", e);
                        None
                    }
                    Err(_) => {
                        crate::log_warn!("[Community] CF 搜索超时 ({}s)", PLATFORM_TIMEOUT_SECS);
                        None
                    }
                },
                None => None,
            }
        },
        async {
            match mr_fut {
                Some(f) => match tokio::time::timeout(
                    std::time::Duration::from_secs(PLATFORM_TIMEOUT_SECS),
                    f,
                ).await {
                    Ok(Ok(r)) => Some(r),
                    Ok(Err(e)) => {
                        crate::log_warn!("[Community] MR 搜索失败: {}", e);
                        None
                    }
                    Err(_) => {
                        crate::log_warn!("[Community] MR 搜索超时 ({}s)", PLATFORM_TIMEOUT_SECS);
                        None
                    }
                },
                None => None,
            }
        },
    );

    let mut projects = Vec::new();
    let mut total = 0u32;

    if let Some((mut cf_projects, cf_total)) = cf_result {
        projects.append(&mut cf_projects);
        total = total.max(cf_total);
    }
    if let Some((mut mr_projects, mr_total)) = mr_result {
        projects.append(&mut mr_projects);
        total = total.max(mr_total);
    }

    if projects.is_empty() {
        return Ok(SearchResult {
            projects: Vec::new(),
            total_count: total,
            page: params.page,
            page_size: PAGE_SIZE,
        });
    }

    // 去重（参考 PCL2 ResourceProject.IsLike）
    let projects = dedup(projects);

    // 排序
    let has_query = !params.query.is_empty();
    let projects = sort_projects(projects, has_query, rtype);

    // 限制单页数量
    let projects: Vec<ResourceProject> = projects
        .into_iter()
        .take(PAGE_SIZE as usize)
        .collect();

    Ok(SearchResult {
        projects,
        total_count: total,
        page: params.page,
        page_size: PAGE_SIZE,
    })
}

/// 跨平台去重（参考 PCL2 ResourceProject.IsLike）
fn dedup(projects: Vec<ResourceProject>) -> Vec<ResourceProject> {
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

/// 排序（参考 PCL2 ResourceSearcher 排序逻辑）
fn sort_projects(
    mut projects: Vec<ResourceProject>,
    has_query: bool,
    rtype: super::types::ResourceType,
) -> Vec<ResourceProject> {
    projects.sort_by(|a, b| {
        let sa = score(a, has_query, rtype);
        let sb = score(b, has_query, rtype);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    projects
}

/// 计算排序分
fn score(p: &ResourceProject, has_query: bool, rtype: super::types::ResourceType) -> f64 {
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

/// 平台下载量权重（参考 PCL2 GetDownloadCountMult）
fn get_download_count_mult(
    platform: super::types::Platform,
    rtype: super::types::ResourceType,
) -> f64 {
    use super::types::{Platform, ResourceType};
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
