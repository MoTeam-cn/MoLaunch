//! 双平台搜索调度器
//!
//! 并行调用 CurseForge 和 Modrinth，合并结果、去重、排序
//!
//! 中文搜索：检测到查询包含中文时，先用本地 moddata.txt 数据库模糊匹配，
//! 把中文关键词重写为英文 Slug/单词后再调平台 API，并对 Modrinth 走 Slug 直查，
//! 绕过两大平台对中文搜索支持不佳的问题（参考 PCL2 `ResourceSearcher.vb`）。

use super::mcmod;
use super::types::{ResourceProject, SearchParams, SearchResult};

/// 每页结果数
pub const PAGE_SIZE: u32 = 40;

/// 单平台搜索超时（秒），防止单个慢平台拖慢整体
const PLATFORM_TIMEOUT_SECS: u64 = 15;

/// 判断查询是否包含中文字符（CJK 统一汉字范围）
fn is_chinese(query: &str) -> bool {
    query.chars().any(|c| {
        let cp = c as u32;
        (0x4E00..=0x9FFF).contains(&cp)
            || (0x3400..=0x4DBF).contains(&cp) // CJK 扩展 A
            || (0xF900..=0xFAFF).contains(&cp) // CJK 兼容 ideographs
    })
}

/// 搜索入口
pub async fn search(params: SearchParams) -> Result<SearchResult, String> {
    let source = params.source; // 0=全部, 1=仅CF, 2=仅MR
    let rtype = params.resource_type;
    let game_version = params.game_version.as_deref();
    let category = params.category.as_deref();
    let original_query = params.query.clone();
    let has_chinese = is_chinese(&original_query);

    // 中文搜索本地映射：检测到中文时，用 moddata.txt 数据库模糊匹配重写查询词
    // 重写后 cf_query / mr_query 可能是英文关键词，也可能为空（未匹配则回退原词）
    // mr_slugs 是 Modrinth Slug 直查列表，用于绕过 MR 搜索 API 的中文限制
    let rewrite = if has_chinese {
        let r = mcmod::search_by_chinese(&original_query);
        crate::log_info!(
            "[Community] 中文搜索拦截: query={}, has_chinese=true, cf_keyword={:?}, mr_keyword={:?}, mr_slugs={}",
            original_query,
            r.cf_keyword,
            r.mr_keyword,
            r.mr_slugs.len()
        );
        r
    } else {
        mcmod::RewriteResult::default()
    };

    // 计算各平台实际使用的查询词
    // - 中文且重写命中：使用重写后的英文关键词
    // - 中文但未命中：回退原词（让平台 API 自己尝试，虽然大概率空结果）
    // - 非中文：原样透传
    let cf_query = if has_chinese {
        rewrite.cf_keyword.unwrap_or_else(|| original_query.clone())
    } else {
        original_query.clone()
    };
    let mr_query = if has_chinese {
        rewrite.mr_keyword.unwrap_or_else(|| original_query.clone())
    } else {
        original_query.clone()
    };
    let mr_slugs = if has_chinese { rewrite.mr_slugs.clone() } else { Vec::new() };

    let mut cf_fut = None;
    let mut mr_fut = None;
    let mut mr_slug_fut = None;

    // 根据来源筛选决定调用哪些平台
    if source == 0 || source == 1 {
        cf_fut = Some(super::curseforge::search(
            &cf_query,
            rtype,
            game_version,
            params.mod_loader,
            category,
            params.page,
        ));
    }
    if source == 0 || source == 2 {
        mr_fut = Some(super::modrinth::search(
            &mr_query,
            rtype,
            game_version,
            params.mod_loader,
            category,
            params.page,
        ));
        // 中文搜索且有 MR Slug 直查列表：并行批量拉取工程详情
        if !mr_slugs.is_empty() {
            mr_slug_fut = Some(super::modrinth::get_projects_by_slugs(&mr_slugs, rtype));
        }
    }

    // 每个平台独立超时，一个慢/失败不阻塞另一个
    // mr_slug_fut（中文 Slug 直查）与 CF/MR 搜索并行执行
    let (cf_result, mr_result, mr_slug_result) = tokio::join!(
        async {
            match cf_fut {
                Some(f) => match tokio::time::timeout(
                    std::time::Duration::from_secs(PLATFORM_TIMEOUT_SECS),
                    f,
                )
                .await
                {
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
                )
                .await
                {
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
        async {
            match mr_slug_fut {
                // Slug 直查不设超时（复用 mr_get 内部的 source 策略和超时控制）
                // get_projects_by_slugs 内部已处理错误，失败返回空 Vec
                Some(f) => Some(f.await),
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
    // 合并 Modrinth Slug 直查结果（中文搜索专用，与 MR 搜索结果去重时由 dedup 处理）
    if let Some(mut mr_slug_projects) = mr_slug_result {
        if !mr_slug_projects.is_empty() {
            crate::log_info!(
                "[Community] MR Slug 直查合并: {} 个工程",
                mr_slug_projects.len()
            );
            // Slug 直查结果的总数未知，不更新 total（避免分页错乱）
            projects.append(&mut mr_slug_projects);
        }
    }

    if projects.is_empty() {
        return Ok(SearchResult {
            projects: Vec::new(),
            total_count: total,
            page: params.page,
            page_size: PAGE_SIZE,
        });
    }

    // 去重
    let projects = dedup(projects);

    // 排序
    let has_query = !params.query.is_empty();
    let projects = sort_projects(projects, has_query, rtype);

    // 限制单页数量
    let projects: Vec<ResourceProject> = projects.into_iter().take(PAGE_SIZE as usize).collect();

    Ok(SearchResult {
        projects,
        total_count: total,
        page: params.page,
        page_size: PAGE_SIZE,
    })
}

/// 跨平台去重
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

/// 排序
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

/// 平台下载量权重
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
