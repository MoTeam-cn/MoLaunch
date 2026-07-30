//! Mod 前置依赖解析器
//!
//! 在用户下载/更新 mod 前，解析该 mod 版本的 dependencies，与本地 mods 目录
//! 比对，返回缺失项（含建议版本）和已满足项。支持递归检查前置的前置（限 3 层），
//! 内置循环依赖防护（visited 集合去重）。
//!
//! 数据来源：`ResourceVersion.dependencies`（platform API 返回的 required project_id
//! 列表，已过滤 Fabric API / Quilt API）。CF 文件级 dependencies 字段在
//! `curseforge/types.rs::CfFile.dependencies` 中读取，MR 在
//! `modrinth/types.rs::MrVersion.dependencies` 中读取。
//!
//! 复用能力：
//! - `modrinth::get_project` / `curseforge::get_project`：查项目详情（含缓存）
//! - `modrinth::get_versions` / `curseforge::get_versions`：查版本列表（含缓存）
//! - `metadata::read_mod_metadata`：读本地 jar 内 slug，用于已安装比对
//! - `helpers::get_mods_dir`：获取版本 mods 目录

use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::minecraft::community::curseforge;
use crate::minecraft::community::modrinth;
use crate::minecraft::community::types::{
    Platform, ReleaseType, ResourceProject, ResourceType, ResourceVersion,
};
use crate::minecraft::download::DownloadSession;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::state::AppState;

/// 递归深度上限（PCL2 不递归，MoLaunch 做一键安装需要处理深层依赖，3 层覆盖 99% 场景）
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
/// 流程：
/// 1. 解析 mods 目录（优先 version_id，其次 mods_dir 参数，都无则跳过已安装扫描）
/// 2. BFS 递归解析 dependencies（限 3 层，visited 集合防环）
///   - 查项目详情（复用 community 缓存）
///   - 检查是否已安装（slug 比对，无 mods 目录时全部视为未安装）
///   - 未安装则选最佳版本（按 game_version + mod_loader 筛选）
///   - 取该版本的 dependencies 继续递归
/// 3. 返回缺失项 + 已满足项
///
/// 单个依赖查询失败不阻断整体流程，log_warn 后跳过。
///
/// # 场景
/// - 版本管理场景：传 version_id，自动解析 mods 目录并扫描已安装
/// - Community 场景：version_id=None + mods_dir=None，跳过已安装扫描，所有前置返回 missing
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
/// 策略（参考 PCL2 隐式策略）：
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

/// 安装结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    /// 成功安装的文件数（含跳过已存在的）
    pub installed_count: u32,
    /// 失败的文件数
    pub failed_count: u32,
    /// 失败详情（文件名：错误原因）
    pub failures: Vec<String>,
}

/// 批量安装主 mod + 用户勾选的前置 mod
///
/// 流程：
/// 1. 解析下载目录（优先 version_id 获取 mods 目录，其次 target_dir 参数）
/// 2. 构造下载任务列表（主 mod + 前置的 suggested_version）
/// 3. 启动 DownloadSession（1 个 stage "下载 Mod 及前置"）
/// 4. download_batch 并发下载，进度推送下载管理页
/// 5. 统计成功/失败，返回 InstallResult
///
/// suggested_version 为 None 的前置记为失败（未找到兼容版本）。
/// download_url 为空的版本记为失败。
///
/// # 场景
/// - 版本管理场景：传 version_id，自动解析 mods 目录
/// - Community 场景：version_id=None + target_dir=Some，下载到用户选择的文件夹
pub async fn install_mod_with_dependencies(
    state: &AppState,
    version_id: Option<&str>,
    target_dir: Option<&str>,
    main_version: &ResourceVersion,
    deps: &[ResolvedDependency],
) -> Result<InstallResult, String> {
    let mods_dir = match (version_id, target_dir) {
        (Some(vid), _) => super::helpers::get_mods_dir(state, vid).await?,
        (None, Some(dir)) => PathBuf::from(dir),
        (None, None) => {
            return Err("必须提供 version_id 或 target_dir 之一".to_string());
        }
    };
    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("创建 mods 目录失败: {}", e))?;
    }

    let mut tasks: Vec<DownloadTask> = Vec::new();
    let mut task_names: Vec<String> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    // 主 mod
    if main_version.download_url.is_empty() {
        failures.push(format!("{}（主 mod 下载地址为空）", main_version.file_name));
    } else {
        let path = mods_dir.join(&main_version.file_name);
        tasks.push(DownloadTask {
            id: format!("mod_main_{}", main_version.file_name),
            urls: crate::minecraft::sources::cdn_urls(&main_version.download_url),
            local_path: path.to_string_lossy().to_string(),
            expected_size: main_version.size as i64,
            expected_hash: main_version.hash.clone(),
        });
        task_names.push(main_version.file_name.clone());
    }

    // 前置 mod
    for dep in deps {
        let Some(ref sv) = dep.suggested_version else {
            failures.push(format!("{}（未找到兼容版本）", dep.project.raw_name));
            continue;
        };
        if sv.download_url.is_empty() {
            failures.push(format!("{}（下载地址为空）", sv.file_name));
            continue;
        }
        let path = mods_dir.join(&sv.file_name);
        tasks.push(DownloadTask {
            id: format!("mod_dep_{}", sv.file_name),
            urls: crate::minecraft::sources::cdn_urls(&sv.download_url),
            local_path: path.to_string_lossy().to_string(),
            expected_size: sv.size as i64,
            expected_hash: sv.hash.clone(),
        });
        task_names.push(sv.file_name.clone());
    }

    let total = tasks.len();
    if total == 0 {
        return Ok(InstallResult {
            installed_count: 0,
            failed_count: failures.len() as u32,
            failures,
        });
    }

    crate::log_info!(
        "[Mods] 开始安装 Mod + 前置：共 {} 个文件（主 mod + {} 个前置）",
        total,
        total - 1
    );

    // 启动 DownloadSession（1 个 stage）
    let session = DownloadSession::start_grouped(
        state,
        "Mod 及前置",
        vec![("下载 Mod 及前置", 100.0)],
    )
    .await;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.version_name = main_version.file_name.clone();
    }

    let progress_callback = session.make_progress_callback(state, 0);
    let results = session
        .manager()
        .download_batch(tasks, Some(progress_callback))
        .await;

    let mut installed_count = 0u32;
    for (name, result) in task_names.iter().zip(results.iter()) {
        if result.status == DownloadStatus::Completed || result.status == DownloadStatus::Skipped {
            installed_count += 1;
        } else {
            let err = result
                .error
                .clone()
                .unwrap_or_else(|| "未知错误".to_string());
            failures.push(format!("{}：{}", name, err));
        }
    }

    let failed_count = total as u32 - installed_count;

    if installed_count > 0 {
        session.mark_complete(state);
    } else {
        session.mark_failed(state, 1);
    }

    crate::log_info!(
        "[Mods] Mod + 前置安装完成：成功 {} / {}，失败 {}",
        installed_count,
        total,
        failed_count
    );

    Ok(InstallResult {
        installed_count,
        failed_count,
        failures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minecraft::community::types::ModLoaders;

    fn make_version(id: &str, gv: &[&str], ml: u32, rt: ReleaseType, date: &str) -> ResourceVersion {
        ResourceVersion {
            id: id.to_string(),
            display: String::new(),
            version: String::new(),
            release_date: date.to_string(),
            download_count: 0,
            mod_loaders: ml,
            game_versions: gv.iter().map(|s| s.to_string()).collect(),
            release_type: rt,
            file_name: String::new(),
            download_url: String::new(),
            hash: None,
            size: 0,
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn test_pick_best_version_prefers_release() {
        let versions = vec![
            make_version("a", &["1.20.1"], ModLoaders::FORGE, ReleaseType::Alpha, "2023-01-01"),
            make_version("b", &["1.20.1"], ModLoaders::FORGE, ReleaseType::Release, "2023-06-01"),
            make_version("c", &["1.20.1"], ModLoaders::FORGE, ReleaseType::Beta, "2023-12-01"),
        ];
        let best = pick_best_version(&versions, "1.20.1", ModLoaders::FORGE);
        assert_eq!(best.unwrap().id, "b");
    }

    #[test]
    fn test_pick_best_version_filters_game_version() {
        let versions = vec![
            make_version("a", &["1.19.2"], ModLoaders::FORGE, ReleaseType::Release, "2023-06-01"),
            make_version("b", &["1.20.1"], ModLoaders::FORGE, ReleaseType::Release, "2023-06-01"),
        ];
        let best = pick_best_version(&versions, "1.20.1", ModLoaders::FORGE);
        assert_eq!(best.unwrap().id, "b");
    }

    #[test]
    fn test_pick_best_version_filters_loader() {
        let versions = vec![
            make_version("a", &["1.20.1"], ModLoaders::FABRIC, ReleaseType::Release, "2023-06-01"),
            make_version("b", &["1.20.1"], ModLoaders::FORGE, ReleaseType::Release, "2023-06-01"),
        ];
        let best = pick_best_version(&versions, "1.20.1", ModLoaders::FORGE);
        assert_eq!(best.unwrap().id, "b");
    }

    #[test]
    fn test_pick_best_version_no_compatible() {
        let versions = vec![
            make_version("a", &["1.19.2"], ModLoaders::FORGE, ReleaseType::Release, "2023-06-01"),
        ];
        let best = pick_best_version(&versions, "1.20.1", ModLoaders::FORGE);
        assert!(best.is_none());
    }

    #[test]
    fn test_is_project_installed_by_slug() {
        let mut installed = HashSet::new();
        installed.insert("jei".to_string());
        let project = ResourceProject {
            platform: Platform::CurseForge,
            resource_type: ResourceType::Mod,
            id: "12345".to_string(),
            slug: "jei".to_string(),
            raw_name: "Just Enough Items".to_string(),
            translated_name: String::new(),
            description: String::new(),
            website: String::new(),
            last_update: String::new(),
            download_count: 0,
            mod_loaders: 0,
            tags: Vec::new(),
            logo_url: None,
            game_versions: Vec::new(),
        };
        assert!(is_project_installed(&project, &installed));
    }

    #[test]
    fn test_is_project_installed_by_id() {
        let mut installed = HashSet::new();
        installed.insert("p7dr8msh".to_string());
        let project = ResourceProject {
            platform: Platform::Modrinth,
            resource_type: ResourceType::Mod,
            id: "P7dR8mSH".to_string(),
            slug: "fabric-api".to_string(),
            raw_name: "Fabric API".to_string(),
            translated_name: String::new(),
            description: String::new(),
            website: String::new(),
            last_update: String::new(),
            download_count: 0,
            mod_loaders: 0,
            tags: Vec::new(),
            logo_url: None,
            game_versions: Vec::new(),
        };
        assert!(is_project_installed(&project, &installed));
    }

    #[test]
    fn test_is_project_installed_not_found() {
        let installed = HashSet::new();
        let project = ResourceProject {
            platform: Platform::CurseForge,
            resource_type: ResourceType::Mod,
            id: "99999".to_string(),
            slug: "unknown-mod".to_string(),
            raw_name: "Unknown".to_string(),
            translated_name: String::new(),
            description: String::new(),
            website: String::new(),
            last_update: String::new(),
            download_count: 0,
            mod_loaders: 0,
            tags: Vec::new(),
            logo_url: None,
            game_versions: Vec::new(),
        };
        assert!(!is_project_installed(&project, &installed));
    }
}
