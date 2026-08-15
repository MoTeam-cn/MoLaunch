use serde::{Deserialize, Serialize};

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

/// 创建房间请求（房主本地生成完整 Scaffolding 码后登记）
///
/// 字段名即 snake_case，与服务端 `CreateRoomRequest` DTO 契约一致。
#[derive(Debug, Clone, Serialize)]
pub struct CreateRoomRequest {
    /// 完整 Scaffolding 房间码 `U/NNNN-NNNN-SSSS-SSSS`
    pub room_code: String,
    /// 房主备注（大厅展示）
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub remark: String,
    /// 是否公开（公开房间进大厅，按整合包聚类）
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub is_public: bool,
    /// 房间密码（空串表示无密码）
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub password: String,
    /// 房主 MC 版本
    pub host_mc_version: String,
    /// 房主 MC 端口（联机中心 hostname 用）
    pub host_mc_port: u16,
    /// 房主加载器类型
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub host_loader: Option<String>,
    /// 房主加载器版本号
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub host_loader_version: Option<String>,
    /// 整合包元数据（`None` = 纯原版房间）
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
    #[serde(alias = "created_at")]
    pub created_at: u64,
    #[serde(alias = "expires_at")]
    pub expires_at: u64,
}

/// 房间公开信息
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfoResponse {
    /// 完整 Scaffolding 房间码（无权限的公开查询可能省略）
    #[serde(default, alias = "room_code")]
    pub room_code: String,
    /// N 段公开标识（大厅展示/去重）
    #[serde(default, alias = "public_identifier")]
    pub public_identifier: String,
    #[serde(alias = "host_device_pk")]
    pub host_device_pk: String,
    /// 是否设置密码
    #[serde(alias = "has_password")]
    pub has_password: bool,
    /// 房主备注
    #[serde(default, alias = "remark")]
    pub remark: String,
    pub status: String,
    #[serde(alias = "created_at")]
    pub created_at: u64,
    #[serde(alias = "expires_at")]
    pub expires_at: u64,
    /// 房主 MC 版本
    #[serde(default, alias = "host_mc_version")]
    pub host_mc_version: String,
    /// 房主 MC 端口
    #[serde(default, alias = "host_mc_port")]
    pub host_mc_port: u16,
    /// 房主加载器类型
    #[serde(default, alias = "host_loader")]
    pub host_loader: Option<String>,
    /// 房主加载器版本号
    #[serde(default, alias = "host_loader_version")]
    pub host_loader_version: Option<String>,
    /// 整合包元数据（`None` = 纯原版房间）
    #[serde(default, alias = "modpack")]
    pub modpack: Option<ModpackMeta>,
}

/// 加入房间响应（join 闸门通过后返回完整码，供房客解析组网）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomResponse {
    /// 完整 Scaffolding 房间码 `U/NNNN-NNNN-SSSS-SSSS`
    #[serde(alias = "room_code")]
    pub room_code: String,
    /// N 段公开标识
    #[serde(default, alias = "public_identifier")]
    pub public_identifier: String,
    /// 房主备注
    #[serde(default, alias = "remark")]
    pub remark: String,
    /// 是否设置密码
    #[serde(alias = "has_password")]
    pub has_password: bool,
    /// 房主 MC 版本
    #[serde(default, alias = "host_mc_version")]
    pub host_mc_version: String,
    /// 房主 MC 端口
    #[serde(default, alias = "host_mc_port")]
    pub host_mc_port: u16,
    /// 房主加载器类型
    #[serde(default, alias = "host_loader")]
    pub host_loader: Option<String>,
    /// 房主加载器版本号
    #[serde(default, alias = "host_loader_version")]
    pub host_loader_version: Option<String>,
    /// 整合包元数据（`None` = 纯原版房间）
    #[serde(default, alias = "modpack")]
    pub modpack: Option<ModpackMeta>,
}

#[cfg(test)]
#[path = "room_tests.rs"]
mod tests;
