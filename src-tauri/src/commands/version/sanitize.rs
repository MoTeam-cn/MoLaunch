//! 版本 ID / MC 版本号合法性校验（防路径遍历）

/// 校验版本 ID / 实例名，防止路径遍历
pub fn sanitize_version_id(id: &str) -> Result<(), String> {
    crate::utils::path::sanitize_file_name(id)?;
    if id.contains(':') {
        return Err(format!("Invalid version id: {}", id));
    }
    // 额外用 components 验证只含 Normal 分量
    let path = std::path::Path::new(id);
    for comp in path.components() {
        if !matches!(comp, std::path::Component::Normal(_)) {
            return Err(format!("Invalid version id: {}", id));
        }
    }
    Ok(())
}

/// 校验 MC 版本号（与 version_id 同样规则）
pub fn sanitize_mc_version(v: &str) -> Result<(), String> {
    sanitize_version_id(v)
}