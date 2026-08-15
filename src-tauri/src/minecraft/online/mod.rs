//! 联机功能模块
//! 对接 MoLaunch API Server 的 P2P 联机能力（认证/信令/TUN 桥接）+ Frp 公共服务
//! （frpc 分发 + 公共 frps 服务器）。协议参考 `api-server/docs/auth.md`、
//! `api-server/docs/signaling.md`、`docs/FRP_PUBLIC_SERVER_API_DESIGN.md`。
//! 安全约束：设备私钥本地持久化；`/v1` 业务请求必须走 ECIES 加密信封。

pub mod auth;
pub mod bridge;
pub mod client;
pub mod client_types;
pub mod crypto;
pub mod ecies;
pub mod frp;
pub mod http_log;
pub mod pow;
pub mod protocol;
pub mod scaffolding;
pub mod signaling;
pub mod storage;
pub mod tun;
