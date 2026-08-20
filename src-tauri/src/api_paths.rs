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

/// 更新清单（GET，`{{target}}/{{arch}}/{{current_version}}/{{channel}}` 由调用方替换）
pub const UPDATES_MANIFEST_RAW: &str = "/v1/updates/manifest/raw?target={{target}}&arch={{arch}}&current_version={{current_version}}&channel={{channel}}";

// ===== v1 FRP =====

/// 公共 frps 服务器列表（GET，直接返回完整连接信息）
pub const FRP_SERVERS: &str = "/v1/frp/servers";

// ===== v1 Signaling（Scaffolding 联机收敛） =====

/// 创建房间（登记完整 Scaffolding 码，POST）
pub const SIGNALING_ROOMS: &str = "/v1/signaling/rooms";
/// 房间信息（GET，`room_code` 可为完整码或 N 段公开标识）
pub const SIGNALING_ROOM: &str = "/v1/signaling/rooms/{room_code}";
/// 加入房间（POST，密码/封禁闸门 → 返回完整码）
pub const SIGNALING_ROOM_JOIN: &str = "/v1/signaling/rooms/{room_code}/join";
/// 房主关闭房间（POST）
pub const SIGNALING_ROOM_CLOSE: &str = "/v1/signaling/rooms/{room_code}/close";
/// 房主心跳上报（POST，每 3 分钟一次，超时未上报由服务端清理）
pub const SIGNALING_ROOM_HEARTBEAT: &str = "/v1/signaling/rooms/{room_code}/heartbeat";
/// 大厅聚合（按整合包分组 + 热度，GET）
pub const SIGNALING_LOBBY_PACKAGES: &str = "/v1/signaling/lobby/packages";
/// 某整合包下的公开房间列表（GET，query 字符串由调用方拼接在常量之后）
pub const SIGNALING_LOBBY_ROOMS: &str = "/v1/signaling/lobby/rooms";
