//! 文件校验补全与启动参数构建
//!
//! 包含 `validate_and_fix_files`（文件完整性检查与补全）和
//! `build_arguments`（委托 `launch::build_launch_arguments` 构建启动参数）。

use std::path::PathBuf;

use crate::log_info;

use super::super::LaunchArguments;
use super::{LaunchError, LaunchPipeline, LaunchStage};

impl LaunchPipeline {
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

        // 使用配置中的参数
        let game_dir = self.config.game_dir.clone();
        let version_id = self.config.version_id.clone();
        let source_mode = crate::minecraft::sources::DownloadSourceMode::Smart;

        // 直接调用异步函数，使用默认参数
        crate::minecraft::download::fix_version_files(
            &version_id,
            &game_dir,
            None, // mirror_url
            8,    // max_threads
            4,    // chunk_count
            0,    // speed_limit
            source_mode,
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
