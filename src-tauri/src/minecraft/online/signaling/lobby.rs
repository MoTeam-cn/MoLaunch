//! 大厅浏览接口（Scaffolding 收敛版）：按整合包聚合的热度列表 + 某整合包下的公开房间列表。

use serde::{Deserialize, Serialize};

use crate::api_paths;
use crate::minecraft::online::client::{BusinessResult, ClientError, OnlineClient};
use crate::minecraft::online::storage::DeviceCredentials;

/// 大厅聚合条目（按整合包分组，字段对齐 api-server `LobbyPackageItem`）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyPackageItem {
    /// 整合包记录主键（UUID）
    #[serde(alias = "modpack_id")]
    pub modpack_id: String,
    /// 整合包名称
    pub name: String,
    /// 来源平台（`curseforge` / `modrinth`）
    pub source: String,
    #[serde(alias = "project_id")]
    pub project_id: String,
    /// 平台文件 ID
    #[serde(alias = "file_id")]
    pub file_id: String,
    /// 整合包对应的 MC 版本
    #[serde(alias = "mc_version")]
    pub mc_version: String,
    /// 整合包自身版本号
    #[serde(default, alias = "modpack_version")]
    pub modpack_version: Option<String>,
    /// 加载器类型
    #[serde(default)]
    pub loader: Option<String>,
    /// 公开房间数（该整合包下）
    #[serde(alias = "room_count")]
    pub room_count: u32,
}

/// 大厅聚合响应（api-server 返回 `{ packages: [...] }`，非分页）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyPackagesResponse {
    pub packages: Vec<LobbyPackageItem>,
    /// 未关联整合包（纯原版等）的公开房间数，前端归类为「其他房间」展示
    #[serde(alias = "other_room_count")]
    pub other_room_count: u64,
}

/// 大厅房间列表查询参数（GET /v1/signaling/lobby/rooms）
///
/// 所有字段可选，未传时服务端使用默认值（package_id 为空时聚合全部公开房间）。
#[derive(Debug, Clone, Default)]
pub struct LobbyListQuery {
    /// 整合包 ID（仅返回该整合包下的公开房间）
    pub package_id: Option<String>,
    /// 页码，默认 1
    pub page: Option<u32>,
    /// 每页数量，默认 20，上限 50
    pub page_size: Option<u32>,
}

/// 大厅房间列表项（摘要，绝不包含 network_secret/完整码）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyRoomItem {
    /// N 段公开标识（去重/入房用，不含密钥；服务端字段为 `code_id`）
    #[serde(alias = "code_id")]
    pub public_identifier: String,
    /// 房主备注
    #[serde(default)]
    pub remark: String,
    /// 是否设置密码
    #[serde(alias = "has_password")]
    pub has_password: bool,
    /// 当前在线人数（房主随心跳上报 easytier peer list 节点数，含房主本机）
    #[serde(default, alias = "current_players")]
    pub player_count: u32,
    #[serde(alias = "max_players")]
    pub max_players: u32,
    /// 房主 MC 版本
    #[serde(default, alias = "host_mc_version")]
    pub host_mc_version: Option<String>,
    #[serde(default, alias = "host_loader")]
    pub host_loader: Option<String>,
    #[serde(alias = "created_at")]
    pub created_at: u64,
    /// 关联整合包摘要（无整合包时为 None，前端归入「其他房间」分组）
    #[serde(default)]
    pub modpack: Option<LobbyModpackSummary>,
}

/// 房间关联的整合包摘要（对齐 api-server `LobbyModpackSummary`，轻量字段）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyModpackSummary {
    #[serde(alias = "modpack_id")]
    pub modpack_id: String,
    pub name: String,
    #[serde(default, alias = "modpack_version")]
    pub modpack_version: Option<String>,
    pub source: String,
    #[serde(alias = "project_id")]
    pub project_id: String,
    #[serde(alias = "file_id")]
    pub file_id: String,
    #[serde(alias = "mc_version")]
    pub mc_version: String,
    #[serde(default)]
    pub loader: Option<String>,
}

/// 大厅房间列表响应（api-server 返回 `{ rooms: [...] }`，非分页）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyListResponse {
    pub rooms: Vec<LobbyRoomItem>,
}

impl OnlineClient {
    /// 查询大厅聚合（GET /v1/signaling/lobby/packages）
    ///
    /// 服务端按整合包分组统计公开房间数并排序热度。
    pub async fn signaling_list_lobby_packages(
        &self,
        creds: &DeviceCredentials,
    ) -> Result<BusinessResult<LobbyPackagesResponse>, ClientError> {
        self.call_v1::<LobbyPackagesResponse>(
            creds,
            "GET",
            api_paths::SIGNALING_LOBBY_PACKAGES,
            None,
            false,
        )
        .await
    }

    /// 查询某整合包下的公开房间列表（GET /v1/signaling/lobby/rooms）
    ///
    /// 列表项仅含公开标识与摘要，完整 Scaffolding 码需走 join 闸门获取。
    pub async fn signaling_list_lobby_rooms(
        &self,
        creds: &DeviceCredentials,
        query: &LobbyListQuery,
    ) -> Result<BusinessResult<LobbyListResponse>, ClientError> {
        // 手动拼接 query string，避免引入 serde_urlencoded 依赖
        let mut pairs: Vec<String> = Vec::new();
        if let Some(ref v) = query.package_id {
            pairs.push(format!("package_id={}", urlencoding::encode(v)));
        }
        if let Some(v) = query.page {
            pairs.push(format!("page={}", v));
        }
        if let Some(v) = query.page_size {
            pairs.push(format!("page_size={}", v));
        }
        let qs = if pairs.is_empty() {
            String::new()
        } else {
            format!("?{}", pairs.join("&"))
        };
        let path = format!("{}{}", api_paths::SIGNALING_LOBBY_ROOMS, qs);
        self.call_v1::<LobbyListResponse>(creds, "GET", &path, None, false)
            .await
    }
}
