//! 信令 action 管理器（房间创建/加入/退出/踢人/保活等）
//!
//! 由 `online_manager::DISPATCHER` 调用 `register_signaling_actions` 注册全部信令 action。
//! 拆分到独立模块避免 `online_manager.rs` 超过 500 行。命名遵循项目 `xxx_manager.rs` 惯例。
//!
//! 所有信令 action 统一流程：
//! 1. 从 params 解析参数
//! 2. 加载设备凭证（需已注册）
//! 3. 调用 `OnlineClient` 对应的 `signaling_*` 方法
//! 4. 返回业务数据

use serde::Deserialize;

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::signaling::CreateRoomRequest;
use crate::minecraft::online::signaling::IceServerEntry;
use crate::minecraft::online::signaling::LobbyListQuery;
use crate::minecraft::online::signaling::ModpackMeta;
use crate::minecraft::online::signaling::UploadParticipantOfferRequest;
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;

// ============================================================
// 参数结构体
// ============================================================

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

// ===== 白名单管理参数（阶段三子任务 8 安全加强） =====

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

// ============================================================
// 辅助函数
// ============================================================

/// 加载设备凭证（需已注册）
///
/// 若 access token 已过期，自动调用 refresh_token 续期；refresh_token 也过期时返回错误。
/// 复用 `online_manager::load_creds_with_auto_refresh`，避免信令 action 各自处理续期逻辑。
async fn load_creds(state: &AppState) -> Result<crate::minecraft::online::storage::DeviceCredentials, String> {
    crate::utils::online_manager::load_creds_with_auto_refresh(state).await
}

/// 创建 OnlineClient
async fn make_client(state: &AppState) -> OnlineClient {
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };
    OnlineClient::new(&base_url)
}

// ============================================================
// 注册入口
// ============================================================

/// 注册全部信令 action 到 dispatcher
pub fn register_signaling_actions(d: &mut Dispatcher) {
    register_get_stun(d);
    register_create_room(d);
    register_get_room(d);
    register_close_room(d);
    register_join_room(d);
    register_submit_answer(d);
    register_list_answers(d);
    register_confirm(d);
    register_keepalive(d);
    // 阶段三子任务 7：房主独占 TURN 拉取接口
    register_get_turn_servers(d);
    register_leave_room(d);
    register_kick(d);
    register_unban(d);
    // 阶段 6.2：房主查询封禁列表
    register_list_bans(d);
    register_list_participants(d);
    register_upload_participant_offer(d);
    register_fetch_participant_offer(d);
    // 阶段三子任务 8：房主白名单管理
    register_list_whitelist(d);
    register_add_whitelist(d);
    register_remove_whitelist(d);
    register_set_whitelist_enabled(d);
    // 联机大厅阶段 5：大厅浏览
    register_list_lobby_rooms(d);
    register_list_lobby_categories(d);
}

// ============================================================
// 各 action 注册
// ============================================================

fn register_get_stun(d: &mut Dispatcher) {
    d.register("room_get_stun", handler!(state, _app, _params, {
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_get_stun");
        let result = client.signaling_get_stun(&creds).await
            .map_err(|e| {
                log_error!("[Online] room_get_stun 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_create_room(d: &mut Dispatcher) {
    d.register("room_create", handler!(state, _app, params, {
        let p: CreateRoomParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!(
            "[Online] room_create: max_players={}, mc_version={}, mc_port={}, loader={:?}, loader_version={:?}, room_type={}, lobby_id={:?}, ice_servers={}, whitelist_enabled={}, whitelist={}, modpack={}",
            p.max_players, p.host_mc_version, p.host_mc_port,
            p.host_loader, p.host_loader_version,
            p.room_type, p.lobby_id,
            p.ice_servers.len(), p.whitelist_enabled, p.whitelist.len(),
            p.modpack.as_ref().map(|m| format!("{}({}:{})", m.source, m.project_id, m.file_id)).unwrap_or_else(|| "none".to_string())
        );
        let req = CreateRoomRequest {
            sdp_offer: p.sdp_offer,
            ice_candidates: p.ice_candidates,
            max_players: p.max_players,
            password: p.password,
            stun_servers: p.stun_servers,
            ice_servers: p.ice_servers,
            host_mc_version: p.host_mc_version,
            host_mc_port: p.host_mc_port,
            host_loader: p.host_loader,
            host_loader_version: p.host_loader_version,
            room_type: p.room_type,
            lobby_id: p.lobby_id,
            whitelist_enabled: p.whitelist_enabled,
            whitelist: p.whitelist,
            modpack: p.modpack,
        };
        let result = client.signaling_create_room(&creds, &req).await
            .map_err(|e| {
                log_error!("[Online] room_create 失败: {}", e);
                e.to_string()
            })?;
        if let Some(ref data) = result.data {
            log_info!("[Online] 房间创建成功: room_code={}", data.room_code);
        }
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_get_room(d: &mut Dispatcher) {
    d.register("room_get", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_get: code={}", p.room_code);
        let result = client.signaling_get_room(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_get 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_close_room(d: &mut Dispatcher) {
    d.register("room_close", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!("[Online] room_close: code={}", p.room_code);
        let result = client.signaling_close_room(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_close 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_join_room(d: &mut Dispatcher) {
    d.register("room_join", handler!(state, _app, params, {
        let p: JoinRoomParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!("[Online] room_join: code={}", p.room_code);
        let result = client.signaling_join_room(&creds, &p.room_code, &p.password).await
            .map_err(|e| {
                log_error!("[Online] room_join 失败: {}", e);
                e.to_string()
            })?;
        if let Some(ref data) = result.data {
            log_info!(
                "[Online] 加入房间成功: participant_id={}, virtual_ip={}",
                data.participant_id, data.player_virtual_ip
            );
        }
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_submit_answer(d: &mut Dispatcher) {
    d.register("room_submit_answer", handler!(state, _app, params, {
        let p: SubmitAnswerParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!(
            "[Online] room_submit_answer: code={}, participant={}",
            p.room_code, p.participant_id
        );
        let result = client
            .signaling_submit_answer(&creds, &p.room_code, &p.participant_id, &p.sdp_answer, &p.ice_candidates)
            .await
            .map_err(|e| {
                log_error!("[Online] room_submit_answer 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_list_answers(d: &mut Dispatcher) {
    d.register("room_list_answers", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        let result = client.signaling_list_answers(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_list_answers 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_confirm(d: &mut Dispatcher) {
    d.register("room_confirm", handler!(state, _app, params, {
        let p: ConfirmParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!(
            "[Online] room_confirm: code={}, participant={}, accepted={}",
            p.room_code, p.participant_id, p.accepted
        );
        let result = client.signaling_confirm(&creds, &p.room_code, &p.participant_id, p.accepted).await
            .map_err(|e| {
                log_error!("[Online] room_confirm 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_keepalive(d: &mut Dispatcher) {
    d.register("room_keepalive", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_keepalive: code={}", p.room_code);
        let result = client.signaling_keepalive(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_keepalive 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

/// 房主独占接口：拉取服务端 TURN 服务器列表（阶段三子任务 7）
///
/// 服务端经负载与启用状态过滤后返回 TURN 服务器数组，
/// 房主拉取后通过 P2P DataChannel 广播 `TurnServers` 控制消息给所有参与者。
fn register_get_turn_servers(d: &mut Dispatcher) {
    d.register("room_get_turn", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_get_turn: code={}", p.room_code);
        let result = client.signaling_get_turn_servers(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_get_turn 失败: {}", e);
                e.to_string()
            })?;
        if let Some(ref data) = result.data {
            log_info!(
                "[Online] 房主拉取 TURN 服务器: enabled={}, servers={}, total_load={}, threshold={}",
                data.enabled, data.servers.len(), data.current_total_load, data.load_threshold
            );
        }
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_leave_room(d: &mut Dispatcher) {
    d.register("room_leave", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!("[Online] room_leave: code={}", p.room_code);
        let result = client.signaling_leave_room(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_leave 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_kick(d: &mut Dispatcher) {
    d.register("room_kick", handler!(state, _app, params, {
        let p: KickParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!(
            "[Online] room_kick: code={}, participant={}, ban={:?}",
            p.room_code, p.participant_id, p.ban_duration_seconds
        );
        let result = client
            .signaling_kick(&creds, &p.room_code, &p.participant_id, p.ban_duration_seconds)
            .await
            .map_err(|e| {
                log_error!("[Online] room_kick 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_unban(d: &mut Dispatcher) {
    d.register("room_unban", handler!(state, _app, params, {
        let p: UnbanParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!("[Online] room_unban: code={}, device_pk={}", p.room_code, p.device_pk);
        let result = client.signaling_unban(&creds, &p.room_code, &p.device_pk).await
            .map_err(|e| {
                log_error!("[Online] room_unban 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_list_bans(d: &mut Dispatcher) {
    d.register("room_list_bans", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_list_bans: code={}", p.room_code);
        let result = client.signaling_list_bans(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_list_bans 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_list_participants(d: &mut Dispatcher) {
    d.register("room_list_participants", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_list_participants: code={}", p.room_code);
        let result = client.signaling_list_participants(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_list_participants 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_upload_participant_offer(d: &mut Dispatcher) {
    d.register("room_upload_participant_offer", handler!(state, _app, params, {
        let p: UploadParticipantOfferParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!(
            "[Online] room_upload_participant_offer: code={}, participant={}",
            p.room_code, p.participant_id
        );
        let req = UploadParticipantOfferRequest {
            sdp_offer: p.sdp_offer,
            ice_candidates: p.ice_candidates,
        };
        let result = client
            .signaling_upload_participant_offer(&creds, &p.room_code, &p.participant_id, &req)
            .await
            .map_err(|e| {
                log_error!("[Online] room_upload_participant_offer 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_fetch_participant_offer(d: &mut Dispatcher) {
    d.register("room_fetch_participant_offer", handler!(state, _app, params, {
        let p: ParticipantOfferParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!(
            "[Online] room_fetch_participant_offer: code={}, participant={}",
            p.room_code, p.participant_id
        );
        let result = client
            .signaling_fetch_participant_offer(&creds, &p.room_code, &p.participant_id)
            .await
            .map_err(|e| {
                log_error!("[Online] room_fetch_participant_offer 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

// ============================================================
// 白名单管理 action（阶段三子任务 8 安全加强）
// ============================================================

fn register_list_whitelist(d: &mut Dispatcher) {
    d.register("room_list_whitelist", handler!(state, _app, params, {
        let p: RoomCodeParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] room_list_whitelist: code={}", p.room_code);
        let result = client.signaling_list_whitelist(&creds, &p.room_code).await
            .map_err(|e| {
                log_error!("[Online] room_list_whitelist 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_add_whitelist(d: &mut Dispatcher) {
    d.register("room_add_whitelist", handler!(state, _app, params, {
        let p: AddWhitelistParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!(
            "[Online] room_add_whitelist: code={}, device_id={}",
            p.room_code, p.device_id
        );
        let result = client
            .signaling_add_whitelist(&creds, &p.room_code, &p.device_id)
            .await
            .map_err(|e| {
                log_error!("[Online] room_add_whitelist 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_remove_whitelist(d: &mut Dispatcher) {
    d.register("room_remove_whitelist", handler!(state, _app, params, {
        let p: RemoveWhitelistParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!(
            "[Online] room_remove_whitelist: code={}, device_id={}",
            p.room_code, p.device_id
        );
        let result = client
            .signaling_remove_whitelist(&creds, &p.room_code, &p.device_id)
            .await
            .map_err(|e| {
                log_error!("[Online] room_remove_whitelist 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_set_whitelist_enabled(d: &mut Dispatcher) {
    d.register("room_set_whitelist_enabled", handler!(state, _app, params, {
        let p: SetWhitelistEnabledParams = serde_json::from_value(params)
            .map_err(|e| format!("参数解析失败: {}", e))?;
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_info!(
            "[Online] room_set_whitelist_enabled: code={}, enabled={}",
            p.room_code, p.enabled
        );
        let result = client
            .signaling_set_whitelist_enabled(&creds, &p.room_code, p.enabled)
            .await
            .map_err(|e| {
                log_error!("[Online] room_set_whitelist_enabled 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

// ============================================================
// 大厅浏览 action（联机大厅阶段 5）
// ============================================================

fn register_list_lobby_rooms(d: &mut Dispatcher) {
    d.register("lobby_list_rooms", handler!(state, _app, params, {
        let p: LobbyListParams = serde_json::from_value(params)
            .unwrap_or_default();
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!(
            "[Online] lobby_list_rooms: lobby={:?}, page={:?}, size={:?}, loader={:?}, keyword={:?}",
            p.lobby_id, p.page, p.page_size, p.loader, p.keyword
        );
        let query = LobbyListQuery {
            lobby_id: p.lobby_id,
            page: p.page,
            page_size: p.page_size,
            has_modpack: p.has_modpack,
            loader: p.loader,
            game_version: p.game_version,
            keyword: p.keyword,
        };
        let result = client.signaling_list_lobby_rooms(&creds, &query).await
            .map_err(|e| {
                log_error!("[Online] lobby_list_rooms 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_list_lobby_categories(d: &mut Dispatcher) {
    d.register("lobby_list_categories", handler!(state, _app, _params, {
        let creds = load_creds(&state).await?;
        let client = make_client(&state).await;
        log_debug!("[Online] lobby_list_categories");
        let result = client.signaling_list_lobby_categories(&creds).await
            .map_err(|e| {
                log_error!("[Online] lobby_list_categories 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}
