//! 房间生命周期接口：创建/查询/加入/关闭（Scaffolding 收敛版）。

use super::types::{CreateRoomRequest, CreateRoomResponse, JoinRoomResponse, RoomInfoResponse};
use crate::api_paths;
use crate::minecraft::online::client::{BusinessResult, ClientError, OnlineClient};
use crate::minecraft::online::storage::DeviceCredentials;

/// 房间码填入路径占位符前先 URL 编码（完整码/公开标识均含 `U/` 前缀，`/` 会拆断路径段）。
fn path_with_room_code(template: &str, room_code: &str) -> String {
    let encoded = urlencoding::encode(room_code);
    template.replace("{room_code}", encoded.as_ref())
}

impl OnlineClient {
    /// 创建房间（登记完整 Scaffolding 码，POST /v1/signaling/rooms）
    pub async fn signaling_create_room(
        &self,
        creds: &DeviceCredentials,
        req: &CreateRoomRequest,
    ) -> Result<BusinessResult<CreateRoomResponse>, ClientError> {
        let body = serde_json::to_value(req)?;
        self.call_v1::<CreateRoomResponse>(
            creds,
            "POST",
            api_paths::SIGNALING_ROOMS,
            Some(&body),
            true,
        )
        .await
    }

    /// 查询房间公开信息（GET /v1/signaling/rooms/{code}）
    ///
    /// `room_code` 可为完整 Scaffolding 码或 N 段公开标识。
    pub async fn signaling_get_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<RoomInfoResponse>, ClientError> {
        let path = path_with_room_code(api_paths::SIGNALING_ROOM, room_code);
        self.call_v1::<RoomInfoResponse>(creds, "GET", &path, None, false)
            .await
    }

    /// 加入房间（POST /v1/signaling/rooms/{code}/join）
    ///
    /// 密码/封禁闸门通过后返回完整 Scaffolding 码，供房客解析组网。
    pub async fn signaling_join_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
        password: &str,
    ) -> Result<BusinessResult<JoinRoomResponse>, ClientError> {
        let path = path_with_room_code(api_paths::SIGNALING_ROOM_JOIN, room_code);
        let body = serde_json::json!({ "password": password });
        self.call_v1::<JoinRoomResponse>(creds, "POST", &path, Some(&body), true)
            .await
    }

    /// 房主关闭房间（POST /v1/signaling/rooms/{code}/close）
    pub async fn signaling_close_room(
        &self,
        creds: &DeviceCredentials,
        room_code: &str,
    ) -> Result<BusinessResult<serde_json::Value>, ClientError> {
        let path = path_with_room_code(api_paths::SIGNALING_ROOM_CLOSE, room_code);
        self.call_v1::<serde_json::Value>(creds, "POST", &path, None, true)
            .await
    }
}
