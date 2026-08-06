//! 多源搜索聚合
//!
//! 并行调用 CurseForge / Modrinth / 中文 Slug 直查，超时隔离并合并结果；排序去重见 sort.rs。

use super::super::mcmod;
use super::super::types::{ResourceProject, SearchParams, SearchResult};
use super::sort::{dedup, sort_projects};

/// 每页结果数
///
/// 取自各平台 API 的每页上限取值：CurseForge 搜索 API 单页最多 50 条、
/// Modrinth 最多 100 条。取 40 为兼容两平台的保守值，避免任一平台因超限报错。
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
    let mr_slugs = if has_chinese {
        rewrite.mr_slugs.clone()
    } else {
        Vec::new()
    };

    let mut cf_fut = None;
    let mut mr_fut = None;
    let mut mr_slug_fut = None;

    // 根据来源筛选决定调用哪些平台
    if source == 0 || source == 1 {
        cf_fut = Some(super::super::curseforge::search(
            &cf_query,
            rtype,
            game_version,
            params.mod_loader,
            category,
            params.page,
        ));
    }
    if source == 0 || source == 2 {
        mr_fut = Some(super::super::modrinth::search(
            &mr_query,
            rtype,
            game_version,
            params.mod_loader,
            category,
            params.page,
        ));
        // 中文搜索且有 MR Slug 直查列表：并行批量拉取工程详情
        if !mr_slugs.is_empty() {
            mr_slug_fut = Some(super::super::modrinth::get_projects_by_slugs(
                &mr_slugs, rtype,
            ));
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
