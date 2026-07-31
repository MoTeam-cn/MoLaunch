//! 房间生命周期接口：STUN 拉取、创建/查询/关闭/加入/退出、保活、TURN 拉取。

use super::types::{
    CreateRoomRequest, CreateRoomResponse, JoinRoomResponse, KeepaliveResponse, RoomInfoResponse,
    StunServersResponse, TurnServersResponse,
};
use crate::minecraft::online::client::{BusinessResult, ClientError, OnlineClient};
use crate::minecraft::online::storage::DeviceCredentials;

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
}
