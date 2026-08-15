//! 信令 action 参数结构体（子模块共用，serde 反序列化）

use crate::minecraft::online::signaling::ModpackMeta;
use serde::Deserialize;

/// `room_create` 参数（房主本地生成完整 Scaffolding 码后登记）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomParams {
    /// 完整 Scaffolding 房间码 `U/NNNN-NNNN-SSSS-SSSS`
    pub room_code: String,
    /// 房主备注（大厅展示）
    #[serde(default)]
    pub remark: String,
    /// 是否公开（公开房间进大厅，按整合包聚类）
    #[serde(default)]
    pub is_public: bool,
    /// 房间密码（空串 = 无密码）
    #[serde(default)]
    pub password: String,
    /// 房主 MC 版本
    #[serde(default)]
    pub host_mc_version: String,
    /// 房主 MC 端口
    #[serde(default)]
    pub host_mc_port: u16,
    /// 房主加载器类型
    #[serde(default)]
    pub host_loader: Option<String>,
    /// 房主加载器版本号
    #[serde(default)]
    pub host_loader_version: Option<String>,
    /// 整合包元数据（`None` = 纯原版房间）
    #[serde(default)]
    pub modpack: Option<ModpackMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomCodeParams {
    /// 房间标识（完整码或 N 段公开标识，由各 action 决定）
    pub room_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomParams {
    pub room_code: String,
    #[serde(default)]
    pub password: String,
}

/// 大厅浏览参数（所有字段可选，未传时服务端使用默认值）
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LobbyListParams {
    /// 整合包 ID（筛选某整合包下的公开房间）
    #[serde(default)]
    pub package_id: Option<String>,
    /// 页码（大厅聚合用）
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub page_size: Option<u32>,
}
