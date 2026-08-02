//! 信令 action 管理器（房间创建/加入/退出/踢人/保活等）
//!
//! 由 `online_manager::DISPATCHER` 调用 `register_signaling_actions` 注册。
//! 流程：解析 params → 加载凭证 → 调用 `OnlineClient::signaling_*` → 返回数据。
//! action 注册按类别拆分到子模块，主文件保留参数结构体与注册入口。

mod lobby_actions;
mod room_actions;
mod session_actions;
mod whitelist_actions;

use serde::Deserialize;

use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::signaling::{IceServerEntry, ModpackMeta};
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;

// 参数结构体

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomParams {
    pub sdp_offer: String,
    #[serde(default)]
    pub ice_candidates: Vec<String>,
    #[serde(default = "default_max_players")]
    pub max_players: u32,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub stun_servers: Vec<String>,
    /// ICE 服务器列表（新客户端优先，可包含 STUN + TURN 凭据）
    ///
    /// 阶段三子任务 7 新增。为空时服务端使用 `stun_servers` 转换为 IceServerEntry 后落库。
    #[serde(default)]
    pub ice_servers: Vec<IceServerEntry>,
    #[serde(default)]
    pub host_mc_version: String,
    #[serde(default)]
    pub host_mc_port: u16,
    /// 房主加载器类型（联机大厅阶段 1 新增）
    ///
    /// 客户端从 `setup.ini` 的 `Type` 字段读取（`forge` / `fabric` / ... / `release`）。
    /// `None` 表示旧客户端未上报，服务端兼容落库为 NULL。
    #[serde(default)]
    pub host_loader: Option<String>,
    /// 房主加载器版本号（联机大厅阶段 1 新增）
    ///
    /// 客户端从 `setup.ini` 的 `ForgeVersion` / `FabricVersion` / ... 字段读取。
    /// 无加载器或 setup.ini 缺失时为 `None`。
    #[serde(default)]
    pub host_loader_version: Option<String>,
    /// 房间类型（联机大厅阶段 2 新增）
    ///
    /// - `private`：仅房间码加入（默认）
    /// - `lobby`：加入大厅，可被大厅浏览页检索到
    #[serde(default = "default_room_type")]
    pub room_type: String,
    /// 大厅 ID（联机大厅阶段 2 新增）
    ///
    /// 仅当 `room_type = "lobby"` 时生效。当前固定为 `global`，
    /// 阶段 5 大厅浏览页支持多大厅选择后扩展。
    #[serde(default)]
    pub lobby_id: Option<String>,
    /// 是否启用白名单（阶段三子任务 8 安全加强）
    ///
    /// `true` 时仅 `whitelist` 数组中的设备可加入；
    /// `true` 且 `whitelist` 为空 = 拒绝所有人加入（仅房主）。
    #[serde(default)]
    pub whitelist_enabled: bool,
    /// 白名单设备 `device_id` 数组（友好标识，服务端转换为 device_pk 落库）
    ///
    /// 仅当 `whitelist_enabled = true` 时生效。房主可在房间运行期间通过
    /// `room_add_whitelist` / `room_remove_whitelist` 动态增删。
    #[serde(default)]
    pub whitelist: Vec<String>,
    /// 整合包元数据（联机大厅阶段 3 新增）
    ///
    /// `None` 表示无整合包（纯原版房间）；`Some` 时服务端 UPSERT 到
    /// `room_modpacks` 表并关联到 rooms.modpack_id。
    /// 前端从 `versions/{id}/modpack.meta.json` 读取后填充。
    #[serde(default)]
    pub modpack: Option<ModpackMeta>,
}

fn default_max_players() -> u32 {
    8
}

/// `CreateRoomParams::room_type` 的默认值（`private`，与 `signaling.rs` 保持一致）
fn default_room_type() -> String {
    "private".to_string()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomCodeParams {
    pub room_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomParams {
    pub room_code: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAnswerParams {
    pub room_code: String,
    pub participant_id: String,
    pub sdp_answer: String,
    #[serde(default)]
    pub ice_candidates: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmParams {
    pub room_code: String,
    pub participant_id: String,
    pub accepted: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KickParams {
    pub room_code: String,
    pub participant_id: String,
    #[serde(default)]
    pub ban_duration_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnbanParams {
    pub room_code: String,
    pub device_pk: String,
}

/// 房主为指定参与者上传 SDP Offer 的参数（mesh 拓扑）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadParticipantOfferParams {
    pub room_code: String,
    pub participant_id: String,
    pub sdp_offer: String,
    #[serde(default)]
    pub ice_candidates: Vec<String>,
}

/// 参与者拉取房主为自己生成的 SDP Offer 的参数（mesh 拓扑）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantOfferParams {
    pub room_code: String,
    pub participant_id: String,
}

/// 添加白名单条目的参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWhitelistParams {
    pub room_code: String,
    /// 待添加的设备 `device_id`（友好标识）
    pub device_id: String,
}

/// 移除白名单条目的参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveWhitelistParams {
    pub room_code: String,
    /// 待移除的设备 `device_id`（友好标识）
    pub device_id: String,
}

/// 修改白名单启用状态的参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWhitelistEnabledParams {
    pub room_code: String,
    pub enabled: bool,
}

/// 大厅房间列表查询参数（联机大厅阶段 5）
///
/// 所有字段可选，未传时服务端使用默认值（lobby_id=global, page=1, page_size=20）。
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LobbyListParams {
    #[serde(default)]
    pub lobby_id: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub page_size: Option<u32>,
    #[serde(default)]
    pub has_modpack: Option<bool>,
    #[serde(default)]
    pub loader: Option<String>,
    #[serde(default)]
    pub game_version: Option<String>,
    #[serde(default)]
    pub keyword: Option<String>,
}

// 辅助函数（子模块共用）

/// 加载设备凭证（需已注册）
///
/// 若 access token 已过期，自动调用 refresh_token 续期；refresh_token 也过期时返回错误。
/// 复用 `online_manager::load_creds_with_auto_refresh`，避免信令 action 各自处理续期逻辑。
async fn load_creds(
    state: &AppState,
) -> Result<crate::minecraft::online::storage::DeviceCredentials, String> {
    crate::commands::online::manager::load_creds_with_auto_refresh(state).await
}

/// 创建 OnlineClient
async fn make_client(state: &AppState) -> OnlineClient {
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };
    OnlineClient::new(&base_url)
}

// 注册入口

/// 注册全部信令 action 到 dispatcher
pub fn register_signaling_actions(d: &mut Dispatcher) {
    room_actions::register(d);
    session_actions::register(d);
    whitelist_actions::register(d);
    lobby_actions::register(d);
}
