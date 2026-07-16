//! 社区资源详情命令

use crate::minecraft::community::curseforge;
use crate::minecraft::community::modrinth;
use crate::minecraft::community::types::{Platform, ResourceProject, ResourceVersion, ResourceType};
use serde::Deserialize;

/// 工程详情请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailRequest {
    pub platform: Platform,
    pub project_id: String,
    pub resource_type: ResourceType,
}

/// 获取工程详情
#[tauri::command]
pub async fn get_project_detail(req: DetailRequest) -> Result<ResourceProject, String> {
    match req.platform {
        Platform::CurseForge => curseforge::get_project(&req.project_id, req.resource_type).await,
        Platform::Modrinth => modrinth::get_project(&req.project_id, req.resource_type).await,
    }
}

/// 获取工程版本列表
#[tauri::command]
pub async fn get_project_versions(
    platform: Platform,
    project_id: String,
) -> Result<Vec<ResourceVersion>, String> {
    match platform {
        Platform::CurseForge => curseforge::get_versions(&project_id).await,
        Platform::Modrinth => modrinth::get_versions(&project_id).await,
    }
}

/// 获取资源的 MC 百科详情页 URL（直链，非搜索页）
///
/// 参考 PCL2 PageDownloadCompDetail.BtnIntroWiki_Click：
/// 通过 moddata.txt 的 slug → 行号（= class id）查表，拼接 `https://www.mcmod.cn/class/<id>.html`
/// 查不到返回 None，前端可回退到搜索 URL
#[tauri::command]
pub async fn get_mcmod_url(platform: Platform, slug: String) -> Result<Option<String>, String> {
    Ok(crate::minecraft::community::mcmod::lookup_class_id(platform, &slug)
        .map(|id| format!("https://www.mcmod.cn/class/{}.html", id)))
}
