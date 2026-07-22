//! 脚本使用的 Java 路径解析（优先用户指定 → 否则按 MC 版本自动检测）

use crate::{log_error, log_info};

/// 解析脚本使用的 Java 路径（优先用户指定 → 否则按 MC 版本自动检测）
/// 用户指定路径会校验版本兼容性，不兼容时返回错误
pub(super) async fn resolve_java_path(
    game_dir: &std::path::Path,
    version_id: &str,
    user_java_path: Option<&str>,
) -> Result<std::path::PathBuf, String> {
    // 获取版本目录和 MC 版本号 + 加载器
    let version_dir = game_dir.join("versions").join(version_id);
    let (mc_version, loader) =
        crate::minecraft::version::setup::detect_version_and_loader(&version_dir, version_id);

    // 1. 优先使用用户指定的 Java 路径（校验版本兼容性）
    if let Some(path) = user_java_path {
        if !path.is_empty() {
            let p = std::path::PathBuf::from(path);
            if p.exists() {
                // 校验版本兼容性
                if let Some(java_ver) = crate::minecraft::java::detect_java_version(path) {
                    if let Err((_cur, cur_min, cur_max)) =
                        crate::minecraft::java_selector::check_java_compatible(
                            java_ver,
                            &mc_version,
                            loader.as_deref(),
                        )
                    {
                        let req_desc = crate::minecraft::java_selector::describe_java_requirement(
                            cur_min, cur_max,
                        );
                        return Err(format!(
                            "Java 版本不兼容：当前版本{}，{}。\n请前往 版本设置 → 游戏 Java 重新选择，或切换为「自动选择」",
                            java_ver, req_desc
                        ));
                    }
                }
                return Ok(p);
            }
            log_error!("User-specified Java not found: {}", path);
        }
    }

    log_info!(
        "[ExportScript] Auto-detecting Java for MC {} (loader: {:?})...",
        mc_version,
        loader
    );

    // 2. 搜索系统 Java
    let java_list = tokio::task::spawn_blocking(crate::minecraft::java::search_java)
        .await
        .map_err(|e| format!("Java 搜索失败: {}", e))?;

    if java_list.is_empty() {
        return Err("未找到任何已安装的 Java，请先安装 Java 或在设置中指定 Java 路径".to_string());
    }

    // 3. 按版本号选择最佳 Java（支持加载器约束）
    let selected = crate::minecraft::java_selector::select_best_java_with_loader(
        &mc_version,
        loader.as_deref(),
        &java_list,
        None,
    )
    .ok_or_else(|| {
        let (min_req, max_req) =
            crate::minecraft::java_selector::get_java_version_range(&mc_version, loader.as_deref());
        format!(
            "未找到满足 MC {} 要求的 Java (需要 Java {}-{})",
            mc_version,
            min_req.unwrap_or(0),
            max_req.map(|m| m.to_string()).unwrap_or("∞".to_string())
        )
    })?;

    Ok(std::path::PathBuf::from(&selected))
}
