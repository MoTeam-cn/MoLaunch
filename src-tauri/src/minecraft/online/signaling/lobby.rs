//! 大厅浏览接口（联机大厅阶段 5）：公开房间列表查询、分类列表查询。
//!
//! 同时定义大厅相关的请求/响应类型。

use serde::{Deserialize, Serialize};

use crate::minecraft::online::client::{BusinessResult, ClientError, OnlineClient};
use crate::minecraft::online::storage::DeviceCredentials;

// 大厅类型

/// 大厅房间列表查询参数
///
/// 对应 `GET /v1/signaling/lobby/rooms` 的 query string。
/// 所有字段均为可选，未传时服务端使用默认值。
#[derive(Debug, Clone, Serialize)]
pub struct LobbyListQuery {
    /// 大厅分类 ID，默认 `global`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lobby_id: Option<String>,
    /// 页码，默认 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// 每页数量，默认 20，上限 50
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// `true` 仅返回有整合包的房间；`false` 仅返回无整合包房间；`None` 不过滤
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_modpack: Option<bool>,
    /// 按房主加载器过滤（`forge` / `fabric` / `neoforge` / `quilt` / `vanilla`）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loader: Option<String>,
    /// 按房主 MC 版本或整合包 MC 版本过滤
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_version: Option<String>,
    /// 模糊匹配房主 MC 版本或整合包名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
}

/// 大厅整合包摘要（列表页轻量版，剔除 `manifest_hash` / `loader_version`）
///
/// 与 `ModpackMeta` 的差异：
/// - 多出 `modpack_id`（服务端主键，详情页可用于去重）
/// - 缺少 `manifest_hash` / `loader_version`（减少列表页载荷）
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LobbyModpackSummary {
    /// 整合包记录主键（UUID）
    #[serde(alias = "modpack_id")]
    pub modpack_id: String,
    pub name: String,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        alias = "modpack_version"
    )]
    pub modpack_version: Option<String>,
    /// 来源平台（`curseforge` / `modrinth`）
    pub source: String,
    #[serde(alias = "project_id")]
    pub project_id: String,
    #[serde(alias = "file_id")]
    pub file_id: String,
    #[serde(alias = "mc_version")]
    pub mc_version: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub loader: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default, alias = "file_size")]
    pub file_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default, alias = "file_count")]
    pub file_count: Option<u32>,
}

/// 大厅房间列表项
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LobbyRoomItem {
    #[serde(alias = "room_code")]
    pub room_code: String,
    #[serde(alias = "host_device_pk")]
    pub host_device_pk: String,
    #[serde(default, alias = "host_mc_version")]
    pub host_mc_version: String,
    #[serde(default, alias = "host_loader")]
    pub host_loader: Option<String>,
    #[serde(default, alias = "host_loader_version")]
    pub host_loader_version: Option<String>,
    #[serde(alias = "max_players")]
    pub max_players: u32,
    #[serde(alias = "player_count")]
    pub player_count: u32,
    #[serde(alias = "has_password")]
    pub has_password: bool,
    pub status: String,
    #[serde(alias = "created_at")]
    pub created_at: u64,
    #[serde(alias = "expires_at")]
    pub expires_at: u64,
    /// 整合包摘要，`None` 表示纯原版房间
    #[serde(default, alias = "modpack")]
    pub modpack: Option<LobbyModpackSummary>,
}

/// 大厅房间列表响应
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyListResponse {
    pub total: u32,
    pub page: u32,
    #[serde(alias = "page_size")]
    pub page_size: u32,
    pub items: Vec<LobbyRoomItem>,
}

/// 大厅分类条目
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LobbyCategory {
    pub id: String,
    pub name: String,
    #[serde(alias = "room_count")]
    pub room_count: u32,
}

/// 大厅分类列表响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LobbyCategoriesResponse {
    pub categories: Vec<LobbyCategory>,
}

// OnlineClient 扩展方法

impl OnlineClient {
    /// 查询大厅公开房间列表（GET /v1/signaling/lobby/rooms）
    ///
    /// 支持分页与过滤（加载器 / MC 版本 / 整合包 / 关键词）。
    /// 列表接口不返回 SDP/ICE/room_key 等敏感字段，加入方需走完整 join 流程。
    pub async fn signaling_list_lobby_rooms(
        &self,
        creds: &DeviceCredentials,
        query: &LobbyListQuery,
    ) -> Result<BusinessResult<LobbyListResponse>, ClientError> {
        // 手动拼接 query string，避免引入 serde_urlencoded 依赖
        let mut pairs: Vec<String> = Vec::new();
        if let Some(ref v) = query.lobby_id {
            pairs.push(format!("lobby_id={}", urlencoding::encode(v)));
        }
        if let Some(v) = query.page {
            pairs.push(format!("page={}", v));
        }
        if let Some(v) = query.page_size {
            pairs.push(format!("page_size={}", v));
        }
        if let Some(v) = query.has_modpack {
            pairs.push(format!("has_modpack={}", v));
        }
        if let Some(ref v) = query.loader {
            pairs.push(format!("loader={}", urlencoding::encode(v)));
        }
        if let Some(ref v) = query.game_version {
            pairs.push(format!("game_version={}", urlencoding::encode(v)));
        }
        if let Some(ref v) = query.keyword {
            pairs.push(format!("keyword={}", urlencoding::encode(v)));
        }
        let qs = if pairs.is_empty() {
            String::new()
        } else {
            format!("?{}", pairs.join("&"))
        };
        let path = format!("/v1/signaling/lobby/rooms{}", qs);
        self.call_v1::<LobbyListResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 查询大厅分类列表（GET /v1/signaling/lobby/categories）
    ///
    /// MVP 阶段仅返回 `global` 一个分类，`room_count` 实时统计。
    pub async fn signaling_list_lobby_categories(
        &self,
        creds: &DeviceCredentials,
    ) -> Result<BusinessResult<LobbyCategoriesResponse>, ClientError> {
        self.call_v1::<LobbyCategoriesResponse>(
            creds,
            "GET",
            "/v1/signaling/lobby/categories",
            None,
            false,
        )
        .await
    }
}
