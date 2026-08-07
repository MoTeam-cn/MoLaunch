//! apiServer HTTP 接口路径集中定义。
//! 统一维护各版本 API、更新、FRP 与信令相关路径，调用方负责填充占位符。

// ===== v3 公共接口（注册/登录/基础鉴权） =====

/// 设备注册（POST）
pub const AUTH_REGISTER: &str = "/v3/auth/register";
/// 设备登录（POST）
pub const AUTH_LOGIN: &str = "/v3/auth/login";
/// 设备登出（POST）
pub const AUTH_LOGOUT: &str = "/v3/auth/logout";
/// access_token 续期（POST）
pub const AUTH_REFRESH: &str = "/v3/auth/refresh";
/// JWKS 公钥集（GET）
pub const JWKS_JSON: &str = "/v3/.well-known/jwks.json";
/// CSRF Token（GET）
pub const CSRF_TOKEN: &str = "/v3/csrf/token";
/// 服务器时间（GET）
pub const TIME: &str = "/v3/time";

// ===== v1 更新 =====

/// 更新清单（GET，`{{target}}/{{arch}}/{{current_version}}` 由调用方替换）
pub const UPDATES_MANIFEST_RAW: &str =
    "/v1/updates/manifest/raw?target={{target}}&arch={{arch}}&current_version={{current_version}}";

// ===== v1 FRP =====

/// frpc/frps 版本清单（GET，组件/平台/架构/当前版本）
pub const FRP_MANIFEST: &str =
    "/v1/frp/manifest?component={component}&target={target}&arch={arch}&current_version={current_version}";
/// 公共 frps 服务器列表（GET）
pub const FRP_SERVERS: &str = "/v1/frp/servers";
/// 分配端口 + per-user token（POST）
pub const FRP_ALLOCATE: &str = "/v1/frp/allocate";
/// 释放分配（POST）
pub const FRP_RELEASE: &str = "/v1/frp/release";
/// 续期分配（POST）
pub const FRP_KEEPALIVE: &str = "/v1/frp/keepalive";

// ===== v1 Signaling =====

/// STUN 服务器列表（GET）
pub const SIGNALING_STUN: &str = "/v1/signaling/stun";
/// 创建房间（POST）
pub const SIGNALING_ROOMS: &str = "/v1/signaling/rooms";
/// 房间公开信息 / 关闭房间（GET / DELETE）
pub const SIGNALING_ROOM: &str = "/v1/signaling/rooms/{room_code}";
/// 加入房间（POST）
pub const SIGNALING_ROOM_JOIN: &str = "/v1/signaling/rooms/{room_code}/join";
/// 房主保活（POST）
pub const SIGNALING_ROOM_KEEPALIVE: &str = "/v1/signaling/rooms/{room_code}/keepalive";
/// TURN 服务器列表（GET，仅房主）
pub const SIGNALING_ROOM_TURN: &str = "/v1/signaling/rooms/{room_code}/turn";
/// 退出房间（DELETE）
pub const SIGNALING_ROOM_PARTICIPANTS_ME: &str = "/v1/signaling/rooms/{room_code}/participants/me";
/// 提交 SDP Answer（POST）
pub const SIGNALING_ROOM_ANSWER: &str = "/v1/signaling/rooms/{room_code}/answer";
/// 待确认 Answer 列表（GET）
pub const SIGNALING_ROOM_ANSWERS: &str = "/v1/signaling/rooms/{room_code}/answers";
/// 确认/拒绝连接（POST）
pub const SIGNALING_ROOM_CONFIRM: &str = "/v1/signaling/rooms/{room_code}/confirm";
/// 踢出参与者（POST）
pub const SIGNALING_ROOM_KICK: &str = "/v1/signaling/rooms/{room_code}/kick";
/// 解封参与者（POST）
pub const SIGNALING_ROOM_UNBAN: &str = "/v1/signaling/rooms/{room_code}/unban";
/// 封禁列表（GET，仅房主）
pub const SIGNALING_ROOM_BANS: &str = "/v1/signaling/rooms/{room_code}/bans";
/// 参与者列表（GET）
pub const SIGNALING_ROOM_PARTICIPANTS: &str = "/v1/signaling/rooms/{room_code}/participants";
/// 房主上传 / 参与者拉取 SDP Offer（PUT / GET）
pub const SIGNALING_ROOM_PARTICIPANT_OFFER: &str =
    "/v1/signaling/rooms/{room_code}/participants/{participant_id}/offer";
/// 白名单列表 / 添加条目（GET / POST）
pub const SIGNALING_ROOM_WHITELIST: &str = "/v1/signaling/rooms/{room_code}/whitelist";
/// 移除白名单条目（DELETE，`device_id` 需 URL 编码）
pub const SIGNALING_ROOM_WHITELIST_REMOVE: &str =
    "/v1/signaling/rooms/{room_code}/whitelist?device_id={device_id}";
/// 白名单启用状态（PATCH）
pub const SIGNALING_ROOM_WHITELIST_ENABLED: &str =
    "/v1/signaling/rooms/{room_code}/whitelist/enabled";
/// 大厅房间列表（GET，query 字符串由调用方拼接在常量之后）
pub const SIGNALING_LOBBY_ROOMS: &str = "/v1/signaling/lobby/rooms";
/// 大厅分类列表（GET）
pub const SIGNALING_LOBBY_CATEGORIES: &str = "/v1/signaling/lobby/categories";
