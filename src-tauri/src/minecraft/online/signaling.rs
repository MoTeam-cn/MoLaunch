//! P2P 联机信令接口客户端
//!
//! 对接 api-server `/v1/signaling/*` 接口，实现房间创建/加入/退出/踢人/保活等。
//!
//! 接口参考：`api-server/docs/signaling.md`
//!
//! 阶段一仅声明类型与接口签名，阶段二填充实现。

use serde::{Deserialize, Serialize};

use super::client::{BusinessResult, ClientError, OnlineClient};
use super::storage::DeviceCredentials;

// ============================== 请求/响应类型 ==============================

/// ICE 服务器条目（对应浏览器 `RTCIceServer` 接口）
///
/// 阶段三子任务 7：STUN 服务器仅填 `urls`，TURN 服务器三个字段都填。
/// 序列化为 JSON 后可直接作为 `RTCPeerConnection` 构造参数的 `iceServers` 数组元素。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct IceServerEntry {
    pub urls: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

impl IceServerEntry {
    /// 从 STUN URL 字符串构造（username/credential 为 None）
    pub fn from_stun_url(url: String) -> Self {
        Self {
            urls: vec![url],
            username: None,
            credential: None,
        }
    }

    /// 从 STUN URL 数组构造（username/credential 为 None）
    pub fn from_stun_urls(urls: Vec<String>) -> Self {
        Self {
            urls,
            username: None,
            credential: None,
        }
    }
}

/// STUN 服务器列表响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StunServersResponse {
    pub servers: Vec<String>,
}

/// TURN 服务器列表响应（房主独占接口，阶段三子任务 7）
///
/// 服务端经负载与启用状态过滤后返回 TURN 服务器数组，
/// 房主拉取后通过 P2P DataChannel 广播 `TurnServers` 控制消息给所有参与者。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnServersResponse {
    /// TURN 服务器条目数组（已过滤禁用/超载的服务器）
    pub servers: Vec<IceServerEntry>,
    /// 服务端是否启用 TURN 中转（false 时 servers 必为空，客户端可降级为纯 STUN）
    pub enabled: bool,
    /// 当前所有启用 TURN 的负载之和（供客户端观测服务端压力）
    #[serde(default, alias = "current_total_load")]
    pub current_total_load: u32,
    /// 服务端配置的负载阈值（0 表示不限制）
    #[serde(default, alias = "load_threshold")]
    pub load_threshold: u32,
}

/// 整合包元数据（联机大厅阶段 3 新增）
///
/// 房主创建房间时关联本地已安装整合包，上报元数据给 api-server。
/// 加入方拉取房间详情后据此判断是否需要一键安装。
///
/// **安全设计**：不包含 `download_url` 字段。加入方通过现有 `getProjectVersions`
/// IPC 反查平台 API 获取下载链接，避免 api-server 成为 URL 分发中心。
///
/// 字段与 api-server `room_modpacks` 表一致（详见 docs/online/lobby-modpack-share.md §3.2）。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModpackMeta {
    /// 来源平台（仅 `curseforge` / `modrinth`）
    pub source: String,
    /// CF project id 或 MR project id
    pub project_id: String,
    /// CF file id 或 MR version id
    pub file_id: String,
    /// 整合包对应的 MC 版本（如 `1.12.2`）
    pub mc_version: String,
    /// 整合包自身版本号（如 `2.9.3`，来自 manifest）
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub modpack_version: Option<String>,
    /// 整合包名称（来自 manifest）
    pub name: String,
    /// 加载器类型（`forge` / `fabric` / `neoforge` / `quilt`）
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub loader: Option<String>,
    /// 加载器版本号
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub loader_version: Option<String>,
    /// 整合包文件大小（字节，仅展示用，来自 manifest）
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub file_size: Option<u64>,
    /// mods 文件数（仅展示用，来自 manifest）
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub file_count: Option<u32>,
    /// manifest.json SHA-256，用于加入方校验本地是否已装同款
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub manifest_hash: Option<String>,
}

/// 创建房间请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateRoomRequest {
    pub sdp_offer: String,
    pub ice_candidates: Vec<String>,
    pub max_players: u32,
    pub password: String,
    pub stun_servers: Vec<String>,
    /// ICE 服务器列表（新客户端优先，可包含 STUN + TURN 凭据）
    ///
    /// 阶段三子任务 7 新增。为空时服务端使用 `stun_servers` 转换为
    /// `IceServerEntry { urls, username: None, credential: None }` 后落库。
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ice_servers: Vec<IceServerEntry>,
    /// 房主 MC 版本（客户端扩展字段，由启动器探测本地 MC 端口后填入）
    pub host_mc_version: String,
    /// 房主 MC 端口（客户端扩展字段，启动器探测本地 Java 进程端口后填入）
    pub host_mc_port: u16,
    /// 房主加载器类型（联机大厅阶段 1 新增）
    ///
    /// 客户端从 `setup.ini` 的 `Type` 字段读取，值为 `forge` / `fabric` / `neoforge` /
    /// `quilt` / `optifine` / `liteloader` / `release` / `snapshot` / `old` / `unknown`。
    /// 服务端可据此在大厅列表展示加载器图标，加入方据此判断兼容性。
    /// `None` 表示旧客户端未上报（兼容字段）。
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub host_loader: Option<String>,
    /// 房主加载器版本号（联机大厅阶段 1 新增）
    ///
    /// 客户端从 `setup.ini` 的 `ForgeVersion` / `FabricVersion` / ... 字段读取，
    /// 如 `47.3.0`。无加载器（原版）或 setup.ini 缺失时为 `None`。
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub host_loader_version: Option<String>,
    /// 房间类型（联机大厅阶段 2 新增）
    ///
    /// - `private`：仅房间码加入（默认，兼容旧客户端）
    /// - `public`：加入大厅，可被大厅浏览页检索到
    ///
    /// 客户端未传时后端默认 `private`，保证旧客户端行为不变。
    /// 注：`CreateRoomRequest` 仅序列化（不反序列化），故无需 `#[serde(default)]`，
    /// 由 `signaling_manager::CreateRoomParams` 反序列化时填默认值。
    #[serde(skip_serializing_if = "std::string::String::is_empty", default)]
    pub room_type: String,
    /// 大厅 ID（联机大厅阶段 2 新增）
    ///
    /// 仅当 `room_type = "lobby"` 时生效，标识房间归属的大厅。
    /// 当前固定为 `global`（全球大厅），后续阶段 5 大厅浏览页支持多大厅选择后扩展。
    /// `private` 房间忽略此字段；`lobby` 房间未传时后端兜底 `global`。
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub lobby_id: Option<String>,
    /// 是否启用白名单（阶段三子任务 8 安全加强）
    ///
    /// `true` 时仅 `whitelist` 数组中的设备可加入；
    /// `true` 且 `whitelist` 为空 = 拒绝所有人加入（仅房主）。
    /// 默认 `false`（不启用白名单，允许任何已注册设备加入）。
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub whitelist_enabled: bool,
    /// 白名单设备 `device_id` 数组（阶段三子任务 8 安全加强）
    ///
    /// 房主创建房间时传入的初始白名单（`device_id` 友好标识）。
    /// 仅当 `whitelist_enabled = true` 时生效；未启用时此字段被忽略。
    /// 房主可在房间运行期间通过 `POST /v1/signaling/rooms/:code/whitelist` 动态增删。
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub whitelist: Vec<String>,
    /// 整合包元数据（联机大厅阶段 3 新增）
    ///
    /// 房主创建房间时关联本地已安装整合包。`None` 表示无整合包（纯原版房间）。
    /// 客户端从 `versions/{id}/modpack.meta.json` 读取后填充此字段。
    /// 不包含 `download_url`，加入方通过现有 IPC 反查平台 API 获取。
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub modpack: Option<ModpackMeta>,
}

/// 创建房间响应
///
/// `rename_all = "camelCase"`：序列化输出 camelCase 给前端
/// `alias`：反序列化时接受 api-server 返回的 snake_case
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomResponse {
    #[serde(alias = "room_code")]
    pub room_code: String,
    #[serde(alias = "host_virtual_ip")]
    pub host_virtual_ip: String,
    pub subnet: String,
    #[serde(alias = "created_at")]
    pub created_at: u64,
    #[serde(alias = "expires_at")]
    pub expires_at: u64,
    /// DataChannel 加密密钥（Base64Url 编码的 32 字节 AES-256 密钥）
    ///
    /// 阶段三子任务 8 新增。空字符串表示服务器未启用加密（兼容旧服务器）。
    #[serde(default, alias = "room_key")]
    pub room_key: String,
}

/// 房间公开信息
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfoResponse {
    #[serde(alias = "room_code")]
    pub room_code: String,
    #[serde(alias = "host_device_pk")]
    pub host_device_pk: String,
    #[serde(alias = "max_players")]
    pub max_players: u32,
    #[serde(alias = "player_count")]
    pub player_count: u32,
    #[serde(alias = "has_password")]
    pub has_password: bool,
    #[serde(alias = "stun_servers")]
    pub stun_servers: Vec<String>,
    /// ICE 服务器列表（统一承载 STUN + TURN，新客户端优先使用）
    ///
    /// 阶段三子任务 7 新增。客户端应优先使用此字段构造 `RTCPeerConnection`，
    /// 为空时回退使用 `stun_servers`。
    #[serde(default, alias = "ice_servers")]
    pub ice_servers: Vec<IceServerEntry>,
    pub status: String,
    #[serde(alias = "created_at")]
    pub created_at: u64,
    #[serde(alias = "expires_at")]
    pub expires_at: u64,
    /// 房主 MC 版本（客户端扩展字段，由创建房间时上报）
    #[serde(default, alias = "host_mc_version")]
    pub host_mc_version: String,
    /// 房主 MC 端口（客户端扩展字段）
    #[serde(default, alias = "host_mc_port")]
    pub host_mc_port: u16,
    /// 是否启用白名单（阶段三子任务 8 安全加强）
    ///
    /// `true` 时仅白名单内设备可加入；`false` 时允许任何已注册设备加入。
    /// 加入方据此判断是否提示房主添加自己到白名单。
    #[serde(default, alias = "whitelist_enabled")]
    pub whitelist_enabled: bool,
    /// 房间类型（联机大厅阶段 2，`private` / `public`，旧服务器缺省 `private`）
    #[serde(default, alias = "room_type")]
    pub room_type: String,
    /// 房主加载器类型（联机大厅阶段 1，如 `forge` / `fabric`，旧服务器缺省 None）
    #[serde(default, alias = "host_loader")]
    pub host_loader: Option<String>,
    /// 房主加载器版本号（联机大厅阶段 1，如 `47.3.0`，旧服务器缺省 None）
    #[serde(default, alias = "host_loader_version")]
    pub host_loader_version: Option<String>,
    /// 整合包元数据（联机大厅阶段 3，`None` 表示纯原版房间）
    ///
    /// 加入方据此判断是否需要一键安装，通过 `check_local_modpack` IPC 校验本地是否已装同款。
    #[serde(default, alias = "modpack")]
    pub modpack: Option<ModpackMeta>,
}

/// 加入房间响应
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomResponse {
    #[serde(alias = "participant_id")]
    pub participant_id: String,
    #[serde(alias = "host_sdp_offer")]
    pub host_sdp_offer: String,
    #[serde(alias = "host_ice_candidates")]
    pub host_ice_candidates: Vec<String>,
    #[serde(alias = "stun_servers")]
    pub stun_servers: Vec<String>,
    /// ICE 服务器列表（与房主一致，统一承载 STUN + TURN）
    ///
    /// 阶段三子任务 7 新增。客户端应优先使用此字段构造 `RTCPeerConnection`，
    /// 为空时回退使用 `stun_servers`。
    #[serde(default, alias = "ice_servers")]
    pub ice_servers: Vec<IceServerEntry>,
    #[serde(alias = "player_virtual_ip")]
    pub player_virtual_ip: String,
    pub subnet: String,
    /// DataChannel 加密密钥（Base64Url 编码的 32 字节 AES-256 密钥，与房主一致）
    ///
    /// 阶段三子任务 8 新增。空字符串表示服务器未启用加密（兼容旧服务器）。
    #[serde(default, alias = "room_key")]
    pub room_key: String,
}

/// 待确认 Answer
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAnswer {
    #[serde(alias = "participant_id")]
    pub participant_id: String,
    #[serde(alias = "device_pk")]
    pub device_pk: String,
    #[serde(alias = "sdp_answer")]
    pub sdp_answer: String,
    #[serde(alias = "ice_candidates")]
    pub ice_candidates: Vec<String>,
    #[serde(alias = "player_virtual_ip")]
    pub player_virtual_ip: String,
    #[serde(alias = "joined_at")]
    pub joined_at: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListAnswersResponse {
    pub answers: Vec<PendingAnswer>,
}

/// 参与者信息
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantInfo {
    #[serde(alias = "participant_id")]
    pub participant_id: String,
    #[serde(alias = "device_pk")]
    pub device_pk: String,
    #[serde(alias = "virtual_ip")]
    pub virtual_ip: String,
    pub status: String,
    #[serde(alias = "joined_at")]
    pub joined_at: u64,
    #[serde(alias = "confirmed_at")]
    pub confirmed_at: u64,
    /// 房主是否已为该参与者生成 SDP Offer（mesh 拓扑，true 表示 offer 已就绪）
    #[serde(default, alias = "host_offer_ready")]
    pub host_offer_ready: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListParticipantsResponse {
    pub participants: Vec<ParticipantInfo>,
}

/// 房间封禁记录（对应 api-server `RoomBan`）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomBan {
    pub id: String,
    #[serde(alias = "room_code")]
    pub room_code: String,
    #[serde(alias = "device_pk")]
    pub device_pk: String,
    /// 0=永久封禁；>0=解封 Unix 秒时间戳
    #[serde(alias = "banned_until")]
    pub banned_until: i64,
    #[serde(alias = "created_at")]
    pub created_at: i64,
}

/// 封禁列表响应（对应 api-server `ListBansResponse`）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBansResponse {
    /// 当前有效封禁记录（永久 + 未过期临时）
    pub bans: Vec<RoomBan>,
    /// 服务端当前 Unix 秒，便于客户端计算剩余封禁时长
    #[serde(alias = "server_time")]
    pub server_time: i64,
}

/// 房主为指定参与者上传 SDP Offer 的请求体（mesh 拓扑）
///
/// 无 `rename_all`：此结构体仅 Serialize（客户端→服务端），服务端期望 snake_case。
/// 若加 `rename_all = "camelCase"` 会序列化为 `sdpOffer`/`iceCandidates`，服务端反序列化失败。
#[derive(Debug, Clone, Serialize)]
pub struct UploadParticipantOfferRequest {
    pub sdp_offer: String,
    pub ice_candidates: Vec<String>,
}

/// 参与者拉取房主为自己生成的 SDP Offer 的响应（mesh 拓扑）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantOfferResponse {
    /// Offer 是否已就绪（等价于 sdp_offer 非空）
    pub ready: bool,
    /// SDP Offer（未就绪时为空字符串）
    #[serde(default, alias = "sdp_offer")]
    pub sdp_offer: String,
    /// ICE Candidates 数组（未就绪时为空数组）
    #[serde(default, alias = "ice_candidates")]
    pub ice_candidates: Vec<String>,
}

/// keepalive 响应
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeepaliveResponse {
    #[serde(alias = "expires_at")]
    pub expires_at: u64,
    #[serde(alias = "server_time")]
    pub server_time: u64,
}

// ============================== 白名单类型（阶段三子任务 8 安全加强） ==============================

/// 白名单条目（房主查询/管理用）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WhitelistEntry {
    /// 设备主键（UUID）
    #[serde(alias = "device_pk")]
    pub device_pk: String,
    /// 设备友好标识（如 `mcsdk-xxxx-xxxx-xxxx-xxxx`）
    #[serde(alias = "device_id")]
    pub device_id: String,
    /// 加入白名单时间（Unix 秒）
    #[serde(alias = "added_at")]
    pub added_at: u64,
}

/// 白名单列表响应
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WhitelistResponse {
    /// 是否启用白名单
    pub enabled: bool,
    /// 白名单条目数组
    pub entries: Vec<WhitelistEntry>,
}

/// 添加白名单请求
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWhitelistRequest {
    /// 待添加的设备 `device_id`（友好标识，服务端转换为 `device_pk` 后落库）
    #[serde(rename = "device_id")]
    pub device_id: String,
}

/// 修改白名单启用状态请求
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWhitelistEnabledRequest {
    /// 是否启用白名单
    pub enabled: bool,
}

// ============================== 大厅类型（联机大厅阶段 5） ==============================

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
    #[serde(skip_serializing_if = "Option::is_none", default, alias = "modpack_version")]
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

// ============================== 客户端扩展方法 ==============================

impl OnlineClient {
    /// 获取 STUN 服务器列表（GET /v1/signaling/stun）
    pub async fn signaling_get_stun(
        &self,
        creds: &DeviceCredentials,
    ) -> Result<BusinessResult<StunServersResponse>, ClientError> {
        self.call_v1::<StunServersResponse>(creds, "GET", "/v1/signaling/stun", None, false)
            .await
    }

    /// 创建房间（POST /v1/signaling/rooms）
    pub async fn signaling_create_room(
        &self,
        creds: &DeviceCredentials,
        req: &CreateRoomRequest,
    ) -> Result<BusinessResult<CreateRoomResponse>, ClientError> {
        let body = serde_json::to_value(req)?;
        self.call_v1::<CreateRoomResponse>(creds, "POST", "/v1/signaling/rooms", Some(&body), true)
            .await
    }

    /// 查询房间公开信息（GET /v1/signaling/rooms/{code}）
    pub async fn signaling_get_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<RoomInfoResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}", room_code);
        self.call_v1::<RoomInfoResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 关闭房间（DELETE /v1/signaling/rooms/{code}）
    pub async fn signaling_close_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}", room_code);
        self.call_v1::<serde_json::Value>(creds, "DELETE", &path, None, true)
            .await
    }

    /// 加入房间（POST /v1/signaling/rooms/{code}/join）
    pub async fn signaling_join_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        password: &str,
    ) -> Result<BusinessResult<JoinRoomResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/join", room_code);
        let body = serde_json::json!({ "password": password });
        self.call_v1::<JoinRoomResponse>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 提交 SDP Answer（POST /v1/signaling/rooms/{code}/answer）
    pub async fn signaling_submit_answer(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
        sdp_answer: &str,
        ice_candidates: &[String],
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/answer", room_code);
        let body = serde_json::json!({
            "participant_id": participant_id,
            "sdp_answer": sdp_answer,
            "ice_candidates": ice_candidates,
        });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 拉取待确认 Answer 列表（GET /v1/signaling/rooms/{code}/answers）
    pub async fn signaling_list_answers(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<ListAnswersResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/answers", room_code);
        self.call_v1::<ListAnswersResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 确认/拒绝连接（POST /v1/signaling/rooms/{code}/confirm）
    pub async fn signaling_confirm(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
        accepted: bool,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/confirm", room_code);
        let body = serde_json::json!({
            "participant_id": participant_id,
            "accepted": accepted,
        });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 房主保活（POST /v1/signaling/rooms/{code}/keepalive）
    pub async fn signaling_keepalive(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<KeepaliveResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/keepalive", room_code);
        self.call_v1::<KeepaliveResponse>(creds, "POST", &path, None, true)
            .await
    }

    /// 房主独占接口：拉取服务端 TURN 服务器列表（GET /v1/signaling/rooms/{code}/turn）
    ///
    /// 阶段三子任务 7：服务端经负载与启用状态过滤后返回 TURN 服务器数组，
    /// 房主拉取后通过 P2P DataChannel 广播 `TurnServers` 控制消息给所有参与者。
    pub async fn signaling_get_turn_servers(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<TurnServersResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/turn", room_code);
        self.call_v1::<TurnServersResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 退出房间（DELETE /v1/signaling/rooms/{code}/participants/me）
    pub async fn signaling_leave_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/participants/me", room_code);
        self.call_v1::<serde_json::Value>(creds, "DELETE", &path, None, true)
            .await
    }

    /// 踢出参与者（POST /v1/signaling/rooms/{code}/kick）
    pub async fn signaling_kick(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
        ban_duration_seconds: Option<u64>,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/kick", room_code);
        let body = serde_json::json!({
            "participant_id": participant_id,
            "ban_duration_seconds": ban_duration_seconds,
        });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 解封参与者（POST /v1/signaling/rooms/{code}/unban）
    pub async fn signaling_unban(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        device_pk: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/unban", room_code);
        let body = serde_json::json!({ "device_pk": device_pk });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 查询房间封禁列表（GET /v1/signaling/rooms/{code}/bans，仅房主）
    ///
    /// 返回当前有效的封禁记录（永久 + 未过期临时），已过期的临时封禁不返回。
    /// 同时返回服务端当前时间 `server_time`，便于客户端计算剩余封禁时长。
    pub async fn signaling_list_bans(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<ListBansResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/bans", room_code);
        self.call_v1::<ListBansResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 查询参与者列表（GET /v1/signaling/rooms/{code}/participants）
    pub async fn signaling_list_participants(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<ListParticipantsResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/participants", room_code);
        self.call_v1::<ListParticipantsResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 房主为指定参与者上传 SDP Offer（PUT /v1/signaling/rooms/{code}/participants/{participant_id}/offer）
    ///
    /// mesh 拓扑：房主轮询 participants 列表发现 `host_offer_ready=false` 的 `joined` 参与者时，
    /// 为其创建独立 PeerConnection + DataChannel + Offer，然后调用本接口上传。
    pub async fn signaling_upload_participant_offer(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
        req: &UploadParticipantOfferRequest,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!(
            "/v1/signaling/rooms/{}/participants/{}/offer",
            room_code, participant_id
        );
        let body = serde_json::to_value(req)?;
        self.call_v1::<serde_json::Value>(creds, "PUT", &path, Some(&body), true)
            .await
    }

    /// 参与者拉取房主为自己生成的 SDP Offer（GET /v1/signaling/rooms/{code}/participants/{participant_id}/offer）
    ///
    /// mesh 拓扑：参与者 join 后轮询本接口，`ready=false` 表示房主尚未生成 Offer。
    pub async fn signaling_fetch_participant_offer(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        participant_id: &str,
    ) -> Result<BusinessResult<ParticipantOfferResponse>, ClientError> {
        let path = format!(
            "/v1/signaling/rooms/{}/participants/{}/offer",
            room_code, participant_id
        );
        self.call_v1::<ParticipantOfferResponse>(creds, "GET", &path, None, false)
            .await
    }

    // ===== 白名单管理（阶段三子任务 8 安全加强） =====

    /// 查询房间白名单（GET /v1/signaling/rooms/{code}/whitelist，仅房主）
    ///
    /// 返回 `enabled` 状态与 `entries` 列表（含 device_pk / device_id / added_at）。
    pub async fn signaling_list_whitelist(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<WhitelistResponse>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/whitelist", room_code);
        self.call_v1::<WhitelistResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 添加白名单条目（POST /v1/signaling/rooms/{code}/whitelist，仅房主，幂等）
    ///
    /// 请求体为 `AddWhitelistRequest { device_id }`，服务端转换为 `device_pk` 后落库。
    /// 重复添加同一 device_id 不会报错（ON CONFLICT DO NOTHING）。
    pub async fn signaling_add_whitelist(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        device_id: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/whitelist", room_code);
        let body = serde_json::json!({ "device_id": device_id });
        self.call_v1::<serde_json::Value>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 移除白名单条目（DELETE /v1/signaling/rooms/{code}/whitelist?device_id=xxx，仅房主）
    ///
    /// 通过 query 参数 `device_id` 指定待移除的设备友好标识。
    /// 严格策略：device_id 找不到设备或不在白名单中都返回错误。
    pub async fn signaling_remove_whitelist(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        device_id: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!(
            "/v1/signaling/rooms/{}/whitelist?device_id={}",
            room_code,
            urlencoding::encode(device_id)
        );
        self.call_v1::<serde_json::Value>(creds, "DELETE", &path, None, true)
            .await
    }

    /// 修改白名单启用状态（PATCH /v1/signaling/rooms/{code}/whitelist/enabled，仅房主）
    ///
    /// 启用白名单但列表为空 = 拒绝所有人加入（仅房主在房间内）。
    /// 关闭白名单后，已加入的参与者不受影响，仅影响后续 join_room 请求。
    pub async fn signaling_set_whitelist_enabled(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        enabled: bool,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = format!("/v1/signaling/rooms/{}/whitelist/enabled", room_code);
        let body = serde_json::json!({ "enabled": enabled });
        // PATCH 方法需要加密信封
        self.call_v1::<serde_json::Value>(creds, "PATCH", &path, Some(&body), true)
            .await
    }

    // ===== 大厅浏览（联机大厅阶段 5） =====

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
        let qs = if pairs.is_empty() { String::new() } else { format!("?{}", pairs.join("&")) };
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
