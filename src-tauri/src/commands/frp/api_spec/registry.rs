//! endpoints.json 规格加载（load_api_spec）

use crate::commands::frp::{providers_root, validate_provider_id, ApiSpec};

/// 读取并解析厂商目录下的 endpoints.json
///
/// 文件位置：`<providers_root>/<provider_id>/<endpoints_file>`
/// endpoints_file 由 manifest.api.endpointsFile 指定，默认 "api/endpoints.json"。
///
/// 校验：provider_id 格式 + 文件存在 + JSON 可解析 + baseUrl 为 HTTPS。
pub fn load_api_spec(provider_id: &str, endpoints_file: &str) -> Result<ApiSpec, String> {
    validate_provider_id(provider_id)?;
    let path = providers_root().join(provider_id).join(endpoints_file);
    if !path.exists() {
        return Err(format!("厂商 endpoints.json 不存在: {}", path.display()));
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("读取 endpoints.json 失败: {}", e))?;
    let spec: ApiSpec =
        serde_json::from_str(&content).map_err(|e| format!("解析 endpoints.json 失败: {}", e))?;

    // 安全：baseUrl 必须为 HTTPS（认证 token 经此通道传输）
    if !spec.base_url.starts_with("https://") {
        return Err(format!(
            "endpoints.json baseUrl 必须使用 HTTPS: {}",
            spec.base_url
        ));
    }

    Ok(spec)
}