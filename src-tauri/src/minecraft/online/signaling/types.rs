//! 信令接口的请求/响应类型定义（ICE、整合包、房间核心类型）
//!
//! 对应 api-server `/v1/signaling/*` 的请求体与响应体结构。

use serde::{Deserialize, Serialize};

// ICE / STUN / TURN

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

// 整合包元数据

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

// 房间核心类型

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

/// keepalive 响应
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeepaliveResponse {
    #[serde(alias = "expires_at")]
    pub expires_at: u64,
    #[serde(alias = "server_time")]
    pub server_time: u64,
}
