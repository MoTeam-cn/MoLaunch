//! yggdrasil HTTP 客户端
//!
//! 封装 yggdrasil API 的 4 个端点：
//! - `POST /authserver/authenticate` 账号密码登录
//! - `POST /authserver/validate`    校验令牌
//! - `POST /authserver/refresh`     刷新令牌
//! - `GET  /`                       服务器元数据
//!
//! 服务器地址（server_url）为 yggdrasil API 根，如 `https://littleskin.cn/api/yggdrasil`。
//! 调用时自动拼接 `/authserver/...` 后缀。
//!
//! URL 规范：authlib-injector.jar 下载源常量统一在 `minecraft::sources` 模块定义；
//! 请求统一走 `crate::http` 模块的 `get_text_with_status` / `post_json_with_status` /
//! `fetch_bytes` 函数，不在此处直接构造 reqwest 请求。

use super::types::{
    ApiError, AuthResponse, AuthenticateRequest, ProfileInfo, RefreshRequest, ServerMetadata,
    SkinCapeInfo, TexturesPayload, ValidateRequest,
};
use crate::minecraft::sources::{
    authlib_injector_meta_url_mirror, authlib_injector_meta_url_official,
};

/// yggdrasil API 错误（含 HTTP 状态码与解析后的错误消息）
#[derive(Debug)]
pub struct YggdrasilError {
    pub status: u16,
    pub message: String,
    /// 是否为网络错误（true 表示无法连接服务器，可重试）
    pub is_network: bool,
}

impl std::fmt::Display for YggdrasilError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_network {
            write!(f, "无法连接到服务器（网络错误）: {}", self.message)
        } else {
            write!(f, "服务器返回错误 ({}): {}", self.status, self.message)
        }
    }
}

impl From<YggdrasilError> for String {
    fn from(e: YggdrasilError) -> String {
        e.to_string()
    }
}

/// 拼接完整 URL（自动去除 server_url 末尾的 `/`）
fn join_url(server_url: &str, path: &str) -> String {
    let base = server_url.trim_end_matches('/');
    format!("{}{}", base, path)
}

/// 解析 yggdrasil 错误响应
fn parse_error(status: u16, body: String) -> YggdrasilError {
    let message = serde_json::from_str::<ApiError>(&body)
        .map(|e| e.message())
        .unwrap_or_else(|_| {
            if body.is_empty() {
                format!("HTTP {}", status)
            } else {
                body.chars().take(200).collect()
            }
        });
    YggdrasilError {
        status,
        message,
        is_network: false,
    }
}

/// GET / 获取服务器元数据
///
/// 用于：
/// 1. 登录页显示服务器名、注册链接
/// 2. 启动游戏时生成 `-Dauthlibinjector.yggdrasil.prefetched` 参数
pub async fn fetch_server_metadata(server_url: &str) -> Result<ServerMetadata, YggdrasilError> {
    let url = join_url(server_url, "");
    // 通过 http.rs 统一入口发起 GET，保留状态码以便区分 200 与错误响应
    let (status, text) = crate::http::get_text_with_status(&url)
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: e.to_string(),
            is_network: true,
        })?;
    if status != 200 {
        return Err(parse_error(status, text));
    }
    serde_json::from_str::<ServerMetadata>(&text).map_err(|e| YggdrasilError {
        status: 0,
        message: format!("解析服务器元数据失败: {}", e),
        is_network: false,
    })
}

/// POST /authserver/authenticate 账号密码登录
pub async fn authenticate(
    server_url: &str,
    username: &str,
    password: &str,
) -> Result<AuthResponse, YggdrasilError> {
    let url = join_url(server_url, "/authserver/authenticate");
    let body = AuthenticateRequest::new(username.to_string(), password.to_string());
    post_json(&url, &body).await
}

/// POST /authserver/validate 校验令牌
///
/// 成功返回 Ok(())，失败返回 Err。
pub async fn validate(
    server_url: &str,
    access_token: &str,
    client_token: Option<&str>,
) -> Result<(), YggdrasilError> {
    let url = join_url(server_url, "/authserver/validate");
    let body = ValidateRequest {
        access_token: access_token.to_string(),
        client_token: client_token.map(|s| s.to_string()),
    };
    // validate 成功返回 204 No Content，与 authenticate/refresh 的 200 不同
    let (status, text) = crate::http::post_json_with_status(&url, &body)
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: e.to_string(),
            is_network: true,
        })?;
    if status == 204 {
        return Ok(());
    }
    Err(parse_error(status, text))
}

/// POST /authserver/refresh 刷新令牌
///
/// 可指定 `selected_profile` 以切换角色（多角色场景）。
pub async fn refresh(
    server_url: &str,
    access_token: &str,
    client_token: Option<&str>,
    selected_profile: Option<super::types::ProfileId>,
) -> Result<AuthResponse, YggdrasilError> {
    let url = join_url(server_url, "/authserver/refresh");
    let body = RefreshRequest {
        access_token: access_token.to_string(),
        client_token: client_token.map(|s| s.to_string()),
        selected_profile,
        request_user: true,
    };
    post_json(&url, &body).await
}

/// 通用 POST JSON 请求（用于 authenticate / refresh）
///
/// 通过 `crate::http::post_json_with_status` 发起请求，保留状态码用于差异化错误处理。
async fn post_json<T: serde::Serialize>(
    url: &str,
    body: &T,
) -> Result<AuthResponse, YggdrasilError> {
    let (status, text) = crate::http::post_json_with_status(url, body)
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: e.to_string(),
            is_network: true,
        })?;
    if status == 200 {
        serde_json::from_str::<AuthResponse>(&text).map_err(|e| YggdrasilError {
            status,
            message: format!("解析响应失败: {}", e),
            is_network: false,
        })
    } else {
        Err(parse_error(status, text))
    }
}

/// 下载 authlib-injector.jar 元数据
///
/// 从官方源 `authlib-injector.yushi.moe` 获取最新版本元数据，
/// 包含 `download_url` 和 `checksums.sha256`。
/// BMCLAPI 作为镜像备用源。
///
/// URL 常量统一在 `minecraft::sources` 模块定义，请求走 `crate::http` 模块。
pub async fn fetch_authlib_injector_meta() -> Result<AuthlibInjectorMeta, String> {
    let primary = authlib_injector_meta_url_official();
    let mirror = authlib_injector_meta_url_mirror();

    let text = match crate::http::fetch_url(&primary).await {
        Ok(t) => t,
        Err(e) => {
            crate::log_info!("[Authlib] 官方源失败，尝试 BMCLAPI 镜像: {}", e);
            crate::http::fetch_url(&mirror)
                .await
                .map_err(|e| format!("获取 authlib-injector 元数据失败: {}", e))?
        }
    };

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析 authlib-injector 元数据失败: {}", e))?;

    let download_url = json["download_url"]
        .as_str()
        .ok_or("元数据缺少 download_url")?
        .to_string();
    let sha256 = json["checksums"]["sha256"]
        .as_str()
        .ok_or("元数据缺少 checksums.sha256")?
        .to_string();

    Ok(AuthlibInjectorMeta {
        download_url,
        sha256,
    })
}

/// authlib-injector.jar 下载元数据
#[derive(Debug, Clone)]
pub struct AuthlibInjectorMeta {
    /// 官方下载地址
    pub download_url: String,
    /// sha256 校验值
    pub sha256: String,
}

/// authlib-injector.jar 在缓存目录的相对路径
///
/// 与 `launch/jvm_args.rs::add_authlib_args` 中常量保持一致。
const AUTHLIB_INJECTOR_JAR_REL: &str = "launch/authlib-injector.jar";

/// 确保 authlib-injector.jar 已下载到缓存目录
///
/// 启动游戏前调用（仅当 `auth_info.server_url` 有值时）。
///
/// 流程：
/// 1. 缓存命中（jar 已存在）→ 直接返回路径，不重复下载
/// 2. 缓存未命中 → fetch_authlib_injector_meta 获取下载 URL 和 sha256
/// 3. 下载 jar 二进制 → 校验 sha256 → 写入缓存
///
/// 同时（可选）预取服务器元数据：当 `server_url` 有值且对应 host 的元数据未缓存时，
/// 调用 `fetch_server_metadata` 拉取并 base64 编码后缓存，供
/// `-Dauthlibinjector.yggdrasil.prefetched` 参数使用。
///
/// 失败时不阻塞启动：返回 Err 由调用方决定是否继续（无外置登录也能进游戏，
/// 只是角色加载/皮肤显示会异常）。
///
/// **阶段 5 改造**：下载方式从 `http::fetch_bytes` 改为 `DownloadManager::download_batch`，
/// 统一走项目下载基础设施（获得限速/URL fallback/进度推送能力）。
/// sha256 校验保持手动实现（DownloadManager 的 expected_hash 用 sha1，与 authlib 的 sha256 不兼容）。
pub async fn ensure_authlib_injector_jar(
    server_url: Option<&str>,
    manager: &crate::minecraft::download::DownloadManager,
) -> Result<std::path::PathBuf, String> {
    // 1. 缓存命中
    if crate::utils::cache::exists(AUTHLIB_INJECTOR_JAR_REL) {
        crate::log_debug!("[Authlib] authlib-injector.jar 缓存命中");
        if let Some(url) = server_url {
            prefetch_metadata_if_missing(url).await;
        }
        return Ok(crate::utils::cache::path(AUTHLIB_INJECTOR_JAR_REL));
    }

    // 2. 获取元数据
    let meta = fetch_authlib_injector_meta().await?;
    crate::log_info!(
        "[Authlib] 准备下载 authlib-injector.jar: url={}, sha256={}",
        meta.download_url,
        &meta.sha256[..8]
    );

    // 3. 通过 DownloadManager 下载到缓存路径（统一限速/URL fallback/进度推送）
    use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
    let target_path = crate::utils::cache::path(AUTHLIB_INJECTOR_JAR_REL);
    // 确保父目录存在（DownloadManager 取消下载时需创建父目录，避免 os error 2）
    if let Some(parent) = target_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let task = DownloadTask {
        id: "authlib_injector".to_string(),
        urls: vec![meta.download_url.clone()],
        local_path: target_path.to_string_lossy().to_string(),
        expected_size: 0, // 不校验 size，下载后手动 sha256 校验
        expected_hash: None, // 不校验 hash，下载后手动 sha256 校验（sha256 与 DownloadManager 的 sha1 不兼容）
    };
    let results = manager.download_batch(vec![task], None).await;
    let result = results.first().ok_or("下载结果为空")?;
    if result.status != DownloadStatus::Completed && result.status != DownloadStatus::Skipped {
        let err = result.error.clone().unwrap_or_else(|| "未知错误".to_string());
        return Err(format!("下载 authlib-injector.jar 失败: {}", err));
    }

    // 4. 读取下载的文件 + 校验 sha256
    let bytes = tokio::fs::read(&target_path)
        .await
        .map_err(|e| format!("读取下载的 authlib-injector.jar 失败: {}", e))?;
    let actual_sha = sha256_hex(&bytes);
    if actual_sha != meta.sha256 {
        // 校验失败：删除损坏的文件
        let _ = std::fs::remove_file(&target_path);
        return Err(format!(
            "authlib-injector.jar sha256 校验失败：期望 {}，实际 {}",
            meta.sha256,
            &actual_sha[..actual_sha.len().min(8)]
        ));
    }

    crate::log_info!("[Authlib] authlib-injector.jar 下载完成 (sha256={})", &meta.sha256[..8]);

    // 5. 预取服务器元数据
    if let Some(url) = server_url {
        prefetch_metadata_if_missing(url).await;
    }

    Ok(target_path)
}

/// 预取服务器元数据并缓存（base64 编码），若对应 host 的缓存已存在则跳过
///
/// 缓存路径：`launch/authlib-prefetched-<host>.txt`
/// 失败时仅打印警告，不阻塞启动（authlib-injector 会在游戏运行时自行拉取）
async fn prefetch_metadata_if_missing(server_url: &str) {
    let host = match extract_host_for_cache(server_url) {
        Some(h) => h,
        None => return,
    };
    let rel = format!("launch/authlib-prefetched-{}.txt", host);
    if crate::utils::cache::exists(&rel) {
        return; // 已缓存
    }

    match fetch_server_metadata(server_url).await {
        Ok(meta) => {
            // 序列化为 JSON 后 base64 编码（authlib-injector 规范要求）
            let json = match serde_json::to_string(&meta) {
                Ok(s) => s,
                Err(e) => {
                    crate::log_warn!("[Authlib] 序列化服务器元数据失败: {}", e);
                    return;
                }
            };
            let b64 = base64_encode(&json.as_bytes());
            if let Err(e) = crate::utils::cache::write(&rel, &b64) {
                crate::log_warn!("[Authlib] 缓存服务器元数据失败: {}", e);
            }
        }
        Err(e) => {
            crate::log_warn!("[Authlib] 预取服务器元数据失败（游戏运行时将自行拉取）: {}", e);
        }
    }
}

/// 计算数据的 sha256 十六进制摘要
///
/// 与 `resources.rs::sha256_hex` 实现等价，此处 inline 避免跨模块 pub 化
/// （sha256_hex 仅 4 行，pub 化会污染 resources 模块 API）
fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// base64 标准编码（不含换行）
fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// 从 server_url 提取 host，用作缓存文件名的安全标识
///
/// 仅保留字母、数字、点、连字符，其他字符替换为 `_`。
/// 与 `launch/jvm_args.rs::extract_host` 保持一致实现。
fn extract_host_for_cache(server_url: &str) -> Option<String> {
    let after_scheme = server_url
        .strip_prefix("https://")
        .or_else(|| server_url.strip_prefix("http://"))
        .unwrap_or(server_url);
    let host_part = after_scheme.split('/').next()?;
    let sanitized: String = host_part
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

// ============================================================
// yggdrasil 皮肤管理端点（参考 yggdrasil-api-analysis.md 4.3 / 4.4 节）
//
// 与认证端点不同，皮肤上传/删除需要 `Authorization: Bearer {accessToken}`
// 和 multipart/form-data，因此不能复用 `crate::http::post_json_with_status`，
// 但仍通过 `crate::http::get_client()` 复用全局 reqwest 客户端（统一 UA / 代理）。
// ============================================================

/// GET /sessionserver/session/minecraft/profile/{uuid} 查询角色属性
///
/// 参考 4.3.1 节：返回完整角色信息（含 properties，textures 为 Base64 编码）。
/// 失败（角色不存在）返回 204 No Content，由调用方转为错误。
pub async fn fetch_profile(server_url: &str, uuid: &str) -> Result<ProfileInfo, YggdrasilError> {
    let path = format!("/sessionserver/session/minecraft/profile/{}", uuid);
    let url = join_url(server_url, &path);
    crate::log_debug!("[Authlib] fetch_profile 请求: {}", url);
    let (status, text) = crate::http::get_text_with_status(&url)
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: e.to_string(),
            is_network: true,
        })?;
    crate::log_debug!(
        "[Authlib] fetch_profile 响应: status={}, body 长度={} 字节",
        status,
        text.len()
    );
    // 204 表示角色不存在（规范定义 4.3.1 失败响应）
    if status == 204 || text.is_empty() {
        crate::log_debug!("[Authlib] fetch_profile 角色不存在 (uuid={})", uuid);
        return Err(YggdrasilError {
            status,
            message: format!("角色 {} 不存在", uuid),
            is_network: false,
        });
    }
    if status != 200 {
        crate::log_debug!(
            "[Authlib] fetch_profile 非 200 响应: status={}, body 前 200 字符={}",
            status,
            text.chars().take(200).collect::<String>()
        );
        return Err(parse_error(status, text));
    }
    let profile = serde_json::from_str::<ProfileInfo>(&text).map_err(|e| {
        crate::log_debug!(
            "[Authlib] fetch_profile 反序列化失败: {}, body 前 200 字符={}",
            e,
            text.chars().take(200).collect::<String>()
        );
        YggdrasilError {
            status: 0,
            message: format!("解析角色属性失败: {}", e),
            is_network: false,
        }
    })?;
    crate::log_debug!(
        "[Authlib] fetch_profile 解析成功: id={}, name={}, properties 数量={}",
        profile.id,
        profile.name,
        profile.properties.len()
    );
    Ok(profile)
}

/// PUT /api/user/profile/{uuid}/skin 上传皮肤
///
/// 参考 4.4.1 节：multipart/form-data，`model` 字段为 "slim" 或空串，
/// `file` 字段为 PNG 二进制（Content-Type: image/png），认证用 Bearer Token。
/// 成功响应 204 No Content。
pub async fn upload_skin(
    server_url: &str,
    access_token: &str,
    uuid: &str,
    png_bytes: Vec<u8>,
    model: &str,
) -> Result<(), YggdrasilError> {
    let path = format!("/api/user/profile/{}/skin", uuid);
    let url = join_url(server_url, &path);

    let form = reqwest::multipart::Form::new()
        .text("model", model.to_string())
        .part(
            "file",
            reqwest::multipart::Part::bytes(png_bytes)
                .file_name("skin.png")
                .mime_str("image/png")
                .map_err(|e| YggdrasilError {
                    status: 0,
                    message: format!("构造 multipart 失败: {}", e),
                    is_network: false,
                })?,
        );

    let client = crate::http::get_client();
    let resp = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: format!("上传皮肤请求失败: {}", e),
            is_network: true,
        })?;

    let status = resp.status().as_u16();
    if status == 204 {
        return Ok(());
    }
    let text = resp.text().await.unwrap_or_default();
    Err(parse_error(status, text))
}

/// DELETE /api/user/profile/{uuid}/skin 删除皮肤
///
/// 参考 4.4.2 节：Bearer 认证，成功 204。
pub async fn delete_skin(
    server_url: &str,
    access_token: &str,
    uuid: &str,
) -> Result<(), YggdrasilError> {
    delete_texture(server_url, access_token, uuid, "skin").await
}

/// PUT /api/user/profile/{uuid}/cape 上传披风
///
/// 参考 4.4.1 节：multipart/form-data，仅含 `file` 字段（PNG），Bearer 认证。
pub async fn upload_cape(
    server_url: &str,
    access_token: &str,
    uuid: &str,
    png_bytes: Vec<u8>,
) -> Result<(), YggdrasilError> {
    let path = format!("/api/user/profile/{}/cape", uuid);
    let url = join_url(server_url, &path);

    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(png_bytes)
            .file_name("cape.png")
            .mime_str("image/png")
            .map_err(|e| YggdrasilError {
                status: 0,
                message: format!("构造 multipart 失败: {}", e),
                is_network: false,
            })?,
    );

    let client = crate::http::get_client();
    let resp = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: format!("上传披风请求失败: {}", e),
            is_network: true,
        })?;

    let status = resp.status().as_u16();
    if status == 204 {
        return Ok(());
    }
    let text = resp.text().await.unwrap_or_default();
    Err(parse_error(status, text))
}

/// DELETE /api/user/profile/{uuid}/cape 删除披风
pub async fn delete_cape(
    server_url: &str,
    access_token: &str,
    uuid: &str,
) -> Result<(), YggdrasilError> {
    delete_texture(server_url, access_token, uuid, "cape").await
}

/// 通用删除材质（skin / cape）
///
/// DELETE /api/user/profile/{uuid}/{textureType}，Bearer 认证，成功 204。
async fn delete_texture(
    server_url: &str,
    access_token: &str,
    uuid: &str,
    texture_type: &str,
) -> Result<(), YggdrasilError> {
    let path = format!("/api/user/profile/{}/{}", uuid, texture_type);
    let url = join_url(server_url, &path);

    let client = crate::http::get_client();
    let resp = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: format!("删除材质请求失败: {}", e),
            is_network: true,
        })?;

    let status = resp.status().as_u16();
    if status == 204 {
        return Ok(());
    }
    let text = resp.text().await.unwrap_or_default();
    Err(parse_error(status, text))
}

/// 从 ProfileInfo 解析出皮肤披风信息
///
/// 遍历 `properties` 数组：
/// - `textures` 属性：Base64 解码为 `TexturesPayload`，提取 SKIN/CAPE URL 与 model
/// - `uploadableTextures` 属性：直接读字符串（"skin" / "cape" / "skin,cape"）
///
/// 解析失败时不阻断流程，仅跳过对应字段（保留默认值）。
pub fn parse_skin_cape_info(profile: &ProfileInfo) -> SkinCapeInfo {
    let mut skin_url = None;
    let mut skin_model = "default".to_string();
    let mut cape_url = None;
    let mut uploadable = String::new();

    crate::log_debug!(
        "[Authlib] parse_skin_cape_info 开始，properties 数量={}",
        profile.properties.len()
    );

    for (idx, prop) in profile.properties.iter().enumerate() {
        crate::log_debug!(
            "[Authlib] properties[{}]: name={}, value 前 80 字符={}",
            idx,
            prop.name,
            prop.value.chars().take(80).collect::<String>()
        );
        match prop.name.as_str() {
            "textures" => {
                match decode_textures(&prop.value) {
                    Some(payload) => {
                        crate::log_debug!(
                            "[Authlib] textures 解码成功: profile_id={}, profile_name={}, skin={:?}, cape={:?}",
                            payload.profile_id,
                            payload.profile_name,
                            payload.textures.skin.as_ref().map(|s| &s.url),
                            payload.textures.cape.as_ref().map(|c| &c.url)
                        );
                        if let Some(skin) = payload.textures.skin {
                            skin_url = Some(skin.url);
                            if let Some(meta) = skin.metadata {
                                if meta.model == "slim" {
                                    skin_model = "slim".to_string();
                                }
                            }
                        }
                        cape_url = payload.textures.cape.map(|c| c.url);
                    }
                    None => {
                        crate::log_debug!(
                            "[Authlib] textures 解码失败（base64 或 JSON 解析失败），value 前 100 字符={}",
                            prop.value.chars().take(100).collect::<String>()
                        );
                    }
                }
            }
            "uploadableTextures" => {
                uploadable = prop.value.clone();
                crate::log_debug!(
                    "[Authlib] uploadableTextures 命中: value={}",
                    uploadable
                );
            }
            _ => {
                crate::log_debug!("[Authlib] 跳过未识别的 property: {}", prop.name);
            }
        }
    }

    let result = SkinCapeInfo {
        skin_url,
        skin_model,
        cape_url,
        uploadable_textures: uploadable,
    };
    crate::log_debug!(
        "[Authlib] parse_skin_cape_info 完成: skin_url={:?}, skin_model={}, cape_url={:?}, uploadable_textures={:?}",
        result.skin_url,
        result.skin_model,
        result.cape_url,
        result.uploadable_textures
    );
    result
}

/// Base64 解码 textures 属性的 value 并反序列化为 TexturesPayload
///
/// 失败返回 None，调用方按"无 textures"处理。
fn decode_textures(b64: &str) -> Option<TexturesPayload> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64.as_bytes())
        .ok()?;
    let json = String::from_utf8(bytes).ok()?;
    crate::log_debug!(
        "[Authlib] decode_textures JSON 解码成功，前 200 字符={}",
        json.chars().take(200).collect::<String>()
    );
    serde_json::from_str(&json).ok()
}
