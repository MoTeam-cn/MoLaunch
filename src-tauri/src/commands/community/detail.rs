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
