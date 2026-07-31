//! 白名单管理接口（阶段三子任务 8 安全加强）：查询/增删/启停。
//!
//! 同时定义白名单相关的请求/响应类型。

use serde::{Deserialize, Serialize};

use crate::minecraft::online::client::{BusinessResult, ClientError, OnlineClient};
use crate::minecraft::online::storage::DeviceCredentials;

// ============================== 白名单类型 ==============================

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

// ============================== OnlineClient 扩展方法 ==============================

impl OnlineClient {
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
}
