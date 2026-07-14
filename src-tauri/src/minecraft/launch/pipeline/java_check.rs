//! Java 检测与校验

use std::path::PathBuf;

use crate::{log_info, log_warn};

use super::{LaunchError, LaunchPipeline, LaunchStage};

impl LaunchPipeline {
    /// 检测Java
    pub(super) async fn detect_java(&self) -> Result<PathBuf, LaunchError> {
        // 获取版本目录
        let version_dir = self
            .config
            .game_dir
            .join("versions")
            .join(&self.config.version_id);

        // 读取 MC 版本号和加载器类型（从 setup.ini 或 JSON）
        let (mc_version, loader) = self.read_mc_version_and_loader(&version_dir);

        // 读取版本 JSON 中的 Mojang 官方 Java 要求
        let mojang_req = self.read_mojang_java_requirement(&version_dir);

        // 计算 Java 版本约束区间 [min, max]
        let (mut min_req, mut max_req) =
            crate::minecraft::java_selector::get_java_version_range(&mc_version, loader.as_deref());

        // Mojang 官方要求覆盖规则表的下限（取更严格者）
        if let Some(mojang_min) = mojang_req {
            min_req = Some(min_req.map_or(mojang_min, |m| m.max(mojang_min)));
        }

        // Java 选择模式（来自 setup.ini 的 JavaMode 字段）
        let java_mode = self.config.java_mode.as_deref().unwrap_or("").trim();
        // auto_version 模式：用用户指定的版本范围覆盖规则表的约束
        if java_mode.eq_ignore_ascii_case("auto_version") {
            if self.config.java_version_min > 0 {
                min_req = Some(self.config.java_version_min);
            }
            // max=0 表示不限上限（清除规则表的上限约束）
            if self.config.java_version_max > 0 {
                max_req = Some(self.config.java_version_max);
            } else {
                max_req = None;
            }
        }

        log_info!(
            "[DetectJava] MC {} (loader: {:?}) requires Java {}-{} (mojang: {:?}, mode: {:?})",
            mc_version,
            loader,
            min_req.unwrap_or(0),
            max_req.map(|m| m.to_string()).unwrap_or("∞".to_string()),
            mojang_req,
            java_mode
        );

        // folder 模式：优先使用版本文件夹下的 Java（runtime/jre 子目录）
        if java_mode.eq_ignore_ascii_case("folder") {
            if let Some(folder_java) = self.find_version_folder_java(&version_dir) {
                log_info!("[DetectJava] Using Java from version folder: {}", folder_java.display());
                return Ok(folder_java);
            }
            log_warn!("[DetectJava] folder 模式未在版本文件夹下找到 Java，回退到自动选择");
        }

        // custom 模式：使用用户指定的 Java
        if java_mode.eq_ignore_ascii_case("custom") {
            if let Some(ref path) = self.config.java_path {
                if !path.is_empty() {
                    let java_path = PathBuf::from(path);
                    if java_path.exists() {
                        // 校验版本兼容性（参考 PCL2：不兼容时阻断启动并提示）
                        if let Some(java_ver) =
                            crate::minecraft::java::detect_java_version(path)
                        {
                            if let Err((_cur, cur_min, cur_max)) =
                                crate::minecraft::java_selector::check_java_compatible(
                                    java_ver,
                                    &mc_version,
                                    loader.as_deref(),
                                )
                            {
                                let req_desc = crate::minecraft::java_selector::describe_java_requirement(cur_min, cur_max);
                                return Err(LaunchError {
                                    stage: LaunchStage::GetJava,
                                    message: format!(
                                        "Java 版本不兼容：当前版本{}，{}。\n请前往 版本设置 → 游戏 Java 重新选择，或切换为「自动选择」",
                                        java_ver, req_desc
                                    ),
                                    is_user_facing: true,
                                });
                            }
                        }
                        return Ok(java_path);
                    }
                    log_warn!("[DetectJava] User-specified Java not found: {}", path);
                }
            }
        }

        self.update_progress(LaunchStage::GetJava, 0.3, "正在搜索系统Java...")
            .await;

        // 搜索Java (使用同步函数在spawn_blocking中运行)
        let mc_version_clone = mc_version.clone();
        let loader_clone = loader.clone();
        let game_dir_clone = self.config.game_dir.clone();
        let java_list = tokio::task::spawn_blocking(move || {
            crate::minecraft::java::search_java_with_paths(&[game_dir_clone])
        })
        .await
        .map_err(|e| LaunchError {
            stage: LaunchStage::GetJava,
            message: format!("Java搜索失败: {}", e),
            is_user_facing: false,
        })?;

        self.update_progress(LaunchStage::GetJava, 0.6, "正在选择最佳Java...")
            .await;

        // 选择最佳Java（支持加载器约束）
        let selected_path = match crate::minecraft::java_selector::select_best_java_with_loader(
            &mc_version_clone,
            loader_clone.as_deref(),
            &java_list,
            None,
        ) {
            Some(path) => path,
            None => {
                // 自动下载补全：用约束区间的下限作为下载目标
                let target_major = min_req.unwrap_or_else(|| {
                    crate::minecraft::java_selector::get_recommended_java_version(&mc_version_clone)
                });
                // 校验推荐版本落在需求区间内
                let in_range = min_req.map_or(true, |m| target_major >= m)
                    && max_req.map_or(true, |m| target_major <= m);
                if !in_range {
                    return Err(LaunchError {
                        stage: LaunchStage::GetJava,
                        message: format!(
                            "未找到满足要求的Java (需要Java {}-{})，且无法自动下载匹配版本。\n请在 版本设置 → 游戏 Java 中手动选择或下载。",
                            min_req.unwrap_or(0),
                            max_req.map(|m| m.to_string()).unwrap_or("∞".to_string())
                        ),
                        is_user_facing: true,
                    });
                }

                self.update_progress(
                    LaunchStage::GetJava,
                    0.7,
                    &format!("未找到兼容 Java，正在自动下载 Java {}...", target_major),
                )
                .await;

                let app_handle = self.config.app_handle.clone();
                let dl_mode =
                    crate::minecraft::sources::DownloadSourceMode::from_str(
                        &self.config.download_source,
                    );
                let mirror_url = self.config.mirror_url.clone();
                let downloaded = crate::minecraft::java::download::download_java_runtime(
                    target_major,
                    dl_mode,
                    mirror_url.as_deref(),
                    app_handle.as_ref(),
                )
                .await
                .map_err(|e| LaunchError {
                    stage: LaunchStage::GetJava,
                    message: format!(
                        "自动下载 Java {} 失败：{}\n请在 版本设置 → 游戏 Java 中手动下载或选择。",
                        target_major, e
                    ),
                    is_user_facing: true,
                })?;

                log_info!("[DetectJava] Auto-downloaded Java: {}", downloaded.display());
                downloaded.to_string_lossy().to_string()
            }
        };

        log_info!("Selected Java: {}", selected_path);
        Ok(PathBuf::from(&selected_path))
    }

    /// 在版本文件夹下查找 Java 可执行文件（folder 模式）
    fn find_version_folder_java(&self, version_dir: &std::path::Path) -> Option<PathBuf> {
        let exe_name = if cfg!(windows) { "javaw.exe" } else { "java" };
        // 候选根目录：runtime/、jre/、java/
        let candidates = ["runtime", "jre", "java"];
        for dir in candidates {
            let root = version_dir.join(dir);
            if !root.exists() {
                continue;
            }
            // 遍历 root 下的所有子目录（包括 root 本身），查找 bin/{exe_name}
            if let Some(found) = Self::search_java_in_dir(&root, exe_name) {
                return Some(found);
            }
        }
        None
    }

    /// 递归查找目录下的 bin/{exe_name}（最多 4 层深度，避免遍历过大）
    fn search_java_in_dir(dir: &std::path::Path, exe_name: &str) -> Option<PathBuf> {
        // 直接检查 dir/bin/{exe_name}
        let direct = dir.join("bin").join(exe_name);
        if direct.exists() {
            return Some(direct);
        }
        // 限制深度为 4 层
        fn walk(dir: &std::path::Path, exe_name: &str, depth: u32) -> Option<PathBuf> {
            if depth > 4 {
                return None;
            }
            let entries = match std::fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => return None,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                // 优先检查 path/bin/{exe_name}
                let candidate = path.join("bin").join(exe_name);
                if candidate.exists() {
                    return Some(candidate);
                }
                if let Some(found) = walk(&path, exe_name, depth + 1) {
                    return Some(found);
                }
            }
            None
        }
        walk(dir, exe_name, 0)
    }

    /// 读取 MC 版本号和加载器类型（从 setup.ini 或 JSON）
    fn read_mc_version_and_loader(&self, version_dir: &std::path::Path) -> (String, Option<String>) {
        crate::minecraft::version::setup::detect_version_and_loader(
            version_dir,
            &self.config.version_id,
        )
    }

    /// 从版本 JSON 读取 Mojang 官方 Java 版本要求
    fn read_mojang_java_requirement(&self, version_dir: &std::path::Path) -> Option<u32> {
        let json_path = version_dir.join(format!("{}.json", self.config.version_id));
        if !json_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&json_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        crate::minecraft::java_selector::get_mojang_java_requirement(&json)
    }
}
