//! Mod 前置依赖解析器
//!
//! BFS 递归解析 mod 版本 dependencies（限 3 层，visited 防环），
//! 与本地 mods 目录比对返回缺失/已满足项，并支持批量安装。
//! 复用 community 缓存查项目/版本，metadata 读本地 slug。

use std::collections::{HashSet, VecDeque};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::minecraft::community::curseforge;
use crate::minecraft::community::modrinth;
use crate::minecraft::community::types::{
    Platform, ReleaseType, ResourceProject, ResourceType, ResourceVersion,
};
use crate::state::AppState;

#[path = "dependency_resolver_install.rs"]
mod install;

pub use install::{install_mod_with_dependencies, InstallResult};

/// 递归深度上限（业界同类启动器通常不递归，MoLaunch 做一键安装需要处理深层依赖，3 层覆盖 99% 场景）
const MAX_DEPTH: u32 = 3;

/// 依赖类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DepType {
    Required,
    Optional,
}

/// 已解析的依赖项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedDependency {
    /// 依赖项目详情
    pub project: ResourceProject,
    /// 依赖类型（当前仅 Required，预留 Optional 扩展）
    pub dependency_type: DepType,
    /// 建议安装的版本（按 game_version + mod_loader 筛选的最佳版本）
    pub suggested_version: Option<ResourceVersion>,
    /// 是否已安装（slug 比对）
    pub is_installed: bool,
    /// 递归深度（0=直接前置，1=前置的前置，…）
    pub depth: u32,
}

/// 依赖检查结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyCheckResult {
    /// 缺失的依赖（需用户确认安装）
    pub missing: Vec<ResolvedDependency>,
    /// 已满足的依赖（本地已安装）
    pub up_to_date: Vec<ResolvedDependency>,
}

/// 检查 mod 版本的前置依赖
///
/// BFS 递归解析 dependencies（限 3 层，visited 防环）：
/// 查项目详情 → 比对本地 slug → 未安装则选最佳版本 → 取其 dependencies 继续递归。
/// 单个依赖查询失败不阻断整体流程，log_warn 后跳过。
pub async fn check_mod_dependencies(
    state: &AppState,
    version_id: Option<&str>,
    mods_dir: Option<&str>,
    platform: Platform,
    root_version: &ResourceVersion,
    game_version: &str,
    mod_loader: u32,
) -> Result<DependencyCheckResult, String> {
    let installed_slugs = match (version_id, mods_dir) {
        (Some(vid), _) => {
            let mods_dir = super::helpers::get_mods_dir(state, vid).await?;
            scan_installed_mod_slugs(&mods_dir)
        }
        (None, Some(dir)) => scan_installed_mod_slugs(Path::new(dir)),
        (None, None) => HashSet::new(),
    };

    crate::log_info!(
        "[Mods] 前置依赖检查：platform={} root={} deps={} game={} loader={} installed={}",
        platform.as_str(),
        root_version.id,
        root_version.dependencies.len(),
        game_version,
        mod_loader,
        installed_slugs.len()
    );

    let mut visited: HashSet<String> = HashSet::new();
    let mut missing: Vec<ResolvedDependency> = Vec::new();
    let mut up_to_date: Vec<ResolvedDependency> = Vec::new();

    // BFS 队列：(平台, project_id, 深度)
    let mut queue: VecDeque<(Platform, String, u32)> = VecDeque::new();
    for dep_id in &root_version.dependencies {
        if visited.insert(dep_id.clone()) {
            queue.push_back((platform, dep_id.clone(), 0));
        }
    }

    while let Some((dep_platform, dep_id, depth)) = queue.pop_front() {
        if depth >= MAX_DEPTH {
            continue;
        }

        // 查项目详情（复用 community 模块缓存）
        let project = match get_project_by_platform(dep_platform, &dep_id).await {
            Ok(p) => p,
            Err(e) => {
                crate::log_warn!(
                    "[Mods] 查询前置 mod 项目失败: {} {} ({})",
                    dep_platform.as_str(),
                    dep_id,
                    e
                );
                continue;
            }
        };

        // 检查是否已安装
        if is_project_installed(&project, &installed_slugs) {
            up_to_date.push(ResolvedDependency {
                project,
                dependency_type: DepType::Required,
                suggested_version: None,
                is_installed: true,
                depth,
            });
            continue;
        }

        // 查版本列表，选最佳版本
        let versions = match get_versions_by_platform(dep_platform, &dep_id).await {
            Ok(v) => v,
            Err(e) => {
                crate::log_warn!(
                    "[Mods] 查询前置 mod 版本列表失败: {} {} ({})",
                    dep_platform.as_str(),
                    dep_id,
                    e
                );
                continue;
            }
        };

        let suggested = pick_best_version(&versions, game_version, mod_loader);

        // 递归：取建议版本的 dependencies 加入队列
        if let Some(ref sv) = suggested {
            for sub_dep in &sv.dependencies {
                if visited.insert(sub_dep.clone()) {
                    queue.push_back((dep_platform, sub_dep.clone(), depth + 1));
                }
            }
        }

        missing.push(ResolvedDependency {
            project,
            dependency_type: DepType::Required,
            suggested_version: suggested,
            is_installed: false,
            depth,
        });
    }

    crate::log_info!(
        "[Mods] 前置依赖检查完成：缺失 {} 项，已满足 {} 项",
        missing.len(),
        up_to_date.len()
    );

    Ok(DependencyCheckResult { missing, up_to_date })
}

/// 扫描 mods 目录，读取所有 jar 的 slug 集合
///
/// slug 来自 jar 内 metadata（fabric.mod.json 的 id / mods.toml 的 modId / mcmod.info 的 modid）
/// 全部转小写，用于大小写不敏感比对
fn scan_installed_mod_slugs(mods_dir: &Path) -> HashSet<String> {
    let mut slugs = HashSet::new();
    if !mods_dir.exists() {
        return slugs;
    }
    let Ok(entries) = std::fs::read_dir(mods_dir) else {
        return slugs;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let lower = name.to_lowercase();
        if !(lower.ends_with(".jar") || lower.ends_with(".litemod")) {
            continue;
        }
        let meta = super::read_mod_metadata(&path);
        if !meta.slug.is_empty() {
            slugs.insert(meta.slug.to_lowercase());
        }
    }
    slugs
}

/// 判断项目是否已安装
///
/// 比对 project.slug 和 project.id 与本地 slug 集合（大小写不敏感）
fn is_project_installed(project: &ResourceProject, installed_slugs: &HashSet<String>) -> bool {
    if !project.slug.is_empty() && installed_slugs.contains(&project.slug.to_lowercase()) {
        return true;
    }
    if !project.id.is_empty() && installed_slugs.contains(&project.id.to_lowercase()) {
        return true;
    }
    false
}

/// 按平台查项目详情
async fn get_project_by_platform(
    platform: Platform,
    id: &str,
) -> Result<ResourceProject, String> {
    match platform {
        Platform::Modrinth => modrinth::get_project(id, ResourceType::Mod).await,
        Platform::CurseForge => curseforge::get_project(id, ResourceType::Mod).await,
    }
}

/// 按平台查版本列表
async fn get_versions_by_platform(
    platform: Platform,
    id: &str,
) -> Result<Vec<ResourceVersion>, String> {
    match platform {
        Platform::Modrinth => modrinth::get_versions(id).await,
        Platform::CurseForge => curseforge::get_versions(id).await,
    }
}

/// 按 game_version + mod_loader 筛选最佳版本
///
/// 策略：
/// 1. 过滤：game_versions 包含目标版本 + mod_loaders 与目标加载器兼容
/// 2. 优先 Release，其次 Beta，最后 Alpha
/// 3. 同优先级选最新（按 release_date 降序）
fn pick_best_version(
    versions: &[ResourceVersion],
    game_version: &str,
    mod_loader: u32,
) -> Option<ResourceVersion> {
    // 过滤兼容版本
    let compatible: Vec<&ResourceVersion> = versions
        .iter()
        .filter(|v| v.game_versions.iter().any(|gv| gv == game_version))
        .filter(|v| mod_loader == 0 || (v.mod_loaders & mod_loader) != 0)
        .collect();

    if compatible.is_empty() {
        return None;
    }

    // 按发布类型分桶，各桶内按 release_date 降序
    let mut releases: Vec<&ResourceVersion> = Vec::new();
    let mut betas: Vec<&ResourceVersion> = Vec::new();
    let mut alphas: Vec<&ResourceVersion> = Vec::new();
    for v in &compatible {
        match v.release_type {
            ReleaseType::Release => releases.push(v),
            ReleaseType::Beta => betas.push(v),
            ReleaseType::Alpha => alphas.push(v),
        }
    }

    // 优先 Release，其次 Beta，最后 Alpha；各桶内取 release_date 最新的
    pick_latest(&releases)
        .or_else(|| pick_latest(&betas))
        .or_else(|| pick_latest(&alphas))
        .cloned()
}

/// 从版本桶中取 release_date 最新的版本
fn pick_latest<'a>(bucket: &[&'a ResourceVersion]) -> Option<&'a ResourceVersion> {
    bucket
        .iter()
        .max_by(|a, b| a.release_date.cmp(&b.release_date))
        .copied()
}

#[cfg(test)]
#[path = "dependency_resolver_tests.rs"]
mod tests;
