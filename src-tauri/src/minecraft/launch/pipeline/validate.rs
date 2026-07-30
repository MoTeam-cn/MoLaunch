//! 文件校验补全与启动参数构建
//!
//! 包含 `validate_and_fix_files`（文件完整性检查与补全）和
//! `build_arguments`（委托 `launch::build_launch_arguments` 构建启动参数）。

use std::path::PathBuf;

use crate::log_info;

use super::super::LaunchArguments;
use super::{LaunchError, LaunchPipeline, LaunchStage};

impl LaunchPipeline {
    /// 从 LaunchConfig 构造 DownloadManager（validate_and_fix_files + build_arguments 复用）
    ///
    /// 阶段 5 提取：消除 validate_and_fix_files 与 build_arguments 中的 manager 构造重复。
    /// 用户设置的 max_threads/chunk_count/speed_limit/download_source 对启动时文件补全
    /// 和 authlib-injector.jar 下载都生效。
    fn download_manager(&self) -> crate::minecraft::download::manager::DownloadManager {
        let download_config = crate::minecraft::download::config::DownloadManagerConfig {
            max_threads: self.config.max_threads as usize,
            chunk_count: self.config.chunk_count as usize,
            speed_limit: self.config.speed_limit,
            source_mode: crate::minecraft::sources::DownloadSourceMode::from_str(
                &self.config.download_source,
            ),
        };
        crate::minecraft::download::manager::DownloadManager::from_config(&download_config)
    }

    /// 检查文件完整性并自动补全
    pub(super) async fn validate_and_fix_files(&self) -> Result<(), LaunchError> {
        let version_dir = self
            .config
            .game_dir
            .join("versions")
            .join(&self.config.version_id);
        let json_path = version_dir.join(format!("{}.json", self.config.version_id));

        // 检查版本是否存在
        if !json_path.exists() {
            return Err(LaunchError {
                stage: LaunchStage::ValidateFiles,
                message: format!("版本 {} 不存在", self.config.version_id),
                is_user_facing: true,
            });
        }

        // 版本独立设置 advance_disable_assets_verify：跳过文件校验和补全
        // 完全不更改 assets；不校验 libraries、第三方登录库与版本主 jar 文件
        if self.config.disable_assets_verify {
            log_info!("[ValidateFiles] disable_assets_verify=true，跳过文件校验和补全");
            self.update_progress(LaunchStage::ValidateFiles, 1.0, "已跳过文件校验")
                .await;
            return Ok(());
        }

        self.update_progress(LaunchStage::ValidateFiles, 0.2, "正在读取版本信息...")
            .await;

        // 读取版本JSON
        let _json_content =
            tokio::fs::read_to_string(&json_path)
                .await
                .map_err(|e| LaunchError {
                    stage: LaunchStage::ValidateFiles,
                    message: format!("读取版本JSON失败: {}", e),
                    is_user_facing: false,
                })?;

        self.update_progress(LaunchStage::ValidateFiles, 0.4, "正在检查并补全文件...")
            .await;

        // 用 LaunchConfig 中的下载参数构造 DownloadManager（build_launch_config 已从全局 config 填充）
        // 替代之前硬编码的 8/4/0/Smart，用户设置的限速/分片/线程数现在对启动时文件补全也生效
        let manager = self.download_manager();

        crate::minecraft::download::fix_version_files(
            &self.config.version_id,
            &self.config.game_dir,
            self.config.mirror_url.as_deref(),
            &manager,
        )
        .await
        .map_err(|e| LaunchError {
            stage: LaunchStage::ValidateFiles,
            message: format!("文件补全失败: {}", e),
            is_user_facing: true,
        })?;

        self.update_progress(LaunchStage::ValidateFiles, 0.9, "文件补全完成")
            .await;

        Ok(())
    }

    /// 构建启动参数
    pub(super) async fn build_arguments(
        &self,
        java_path: &PathBuf,
    ) -> Result<LaunchArguments, LaunchError> {
        // 外置登录（authlib-injector）：确保 authlib-injector.jar 已下载到缓存
        // 仅当 auth_info.server_url 有值时执行。失败不阻塞启动，
        // add_authlib_args 内部会检测 jar 是否存在并打印警告。
        // 阶段 5：通过 DownloadManager 下载（统一限速/URL fallback），与 validate_and_fix_files 复用 manager 构造
        if let Some(ref server_url) = self.config.auth_info.server_url {
            if !server_url.is_empty() {
                let manager = self.download_manager();
                let _ = crate::minecraft::auth::authlib::ensure_authlib_injector_jar(
                    Some(server_url),
                    &manager,
                )
                .await;
            }
        }

        super::super::build_launch_arguments(
            &self.config.game_dir,
            &self.config.version_id,
            java_path,
            &self.config.auth_info,
            self.config.min_memory,
            self.config.max_memory,
            self.config.window_width,
            self.config.window_height,
            self.config.server_address.as_deref(),
            self.config.server_port,
            self.config.isolation_mode,
            &self.config.extra_jvm_args,
            &self.config.extra_game_args,
            self.config.disable_jlw,
            self.config.disable_lua,
            self.config.custom_info.as_deref(),
            self.config.game_language.as_deref(),
        )
        .map_err(|e| LaunchError {
            stage: LaunchStage::BuildArgs,
            message: format!("构建参数失败: {}", e),
            is_user_facing: false,
        })
    }
}
