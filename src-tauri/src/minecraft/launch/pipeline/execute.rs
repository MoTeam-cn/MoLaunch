//! 启动流程编排
//!
//! 实现 `LaunchPipeline::execute` 主流程与 `update_progress` 进度更新工具方法。

use crate::{log_info, log_warn};

use super::{LaunchError, LaunchPipeline, LaunchResult, LaunchStage};

impl LaunchPipeline {
    /// 更新进度
    pub(super) async fn update_progress(
        &self,
        stage: LaunchStage,
        stage_progress: f64,
        message: impl Into<String>,
    ) {
        let mut progress = self.progress.write().await;
        let stages = vec![
            LaunchStage::GetJava,
            LaunchStage::ValidateFiles,
            LaunchStage::BuildArgs,
            LaunchStage::PreLaunch,
            LaunchStage::ExtractNatives,
            LaunchStage::LaunchProcess,
            LaunchStage::WaitWindow,
        ];
        let total_weight: f64 = stages.iter().map(|s| s.weight()).sum();

        let mut completed_weight = 0.0;
        for s in &stages {
            if *s == stage {
                completed_weight += s.weight() * stage_progress;
                break;
            } else {
                completed_weight += s.weight();
            }
        }

        progress.stage = stage;
        progress.stage_progress = stage_progress;
        progress.overall_progress = completed_weight / total_weight;
        progress.message = message.into();
    }

    /// 执行启动流程
    pub async fn execute(&self) -> Result<LaunchResult, LaunchError> {
        log_info!(
            "Starting launch pipeline for version: {}",
            self.config.version_id
        );

        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::Init,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }

        // 阶段1: 获取Java
        self.update_progress(LaunchStage::GetJava, 0.0, "正在检测Java...")
            .await;
        let java_path = self.detect_java().await?;
        self.update_progress(LaunchStage::GetJava, 1.0, "Java检测完成")
            .await;

        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::GetJava,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }

        // 阶段2: 文件检查和补全
        self.update_progress(LaunchStage::ValidateFiles, 0.0, "正在检查游戏文件...")
            .await;
        self.validate_and_fix_files().await?;
        self.update_progress(LaunchStage::ValidateFiles, 1.0, "文件检查完成")
            .await;

        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::ValidateFiles,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }

        // 阶段3: 构建参数（内包含语言设置）
        self.update_progress(LaunchStage::BuildArgs, 0.0, "正在构建启动参数...")
            .await;
        let launch_args = self.build_arguments(&java_path).await?;
        self.update_progress(LaunchStage::BuildArgs, 1.0, "参数构建完成")
            .await;

        // 检查取消
        if *self.cancel_flag.lock().await {
            return Err(LaunchError {
                stage: LaunchStage::BuildArgs,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }

        // 阶段4: 启动前命令（advance_run_cmd，参考 PCL2 的 PreLaunch）
        // 高性能显卡设置也在这一阶段执行（参考 PCL2 McLaunchPrerun）
        if self.config.use_dedicated_gpu {
            self.update_progress(LaunchStage::PreLaunch, 0.0, "正在设置高性能显卡...")
                .await;
            if let Err(e) = self.set_gpu_preference(&java_path).await {
                log_warn!("[Launch] 设置高性能显卡失败: {}", e);
            }
        }
        if self.config.pre_launch_cmd.is_some() {
            self.update_progress(LaunchStage::PreLaunch, 0.0, "正在执行启动前命令...")
                .await;
            self.run_pre_launch().await?;
            self.update_progress(LaunchStage::PreLaunch, 1.0, "启动前命令执行完成")
                .await;
        }

        // 阶段5: 解压Natives
        self.update_progress(LaunchStage::ExtractNatives, 0.0, "正在解压原生库...")
            .await;
        self.extract_natives().await?;
        self.update_progress(LaunchStage::ExtractNatives, 1.0, "原生库解压完成")
            .await;

        // 阶段6: 启动进程
        self.update_progress(LaunchStage::LaunchProcess, 0.0, "正在启动游戏...")
            .await;
        let result = self.launch_process(&java_path, &launch_args).await?;
        self.update_progress(
            LaunchStage::LaunchProcess,
            1.0,
            format!("游戏已启动 PID: {}", result.pid),
        )
        .await;

        // 阶段7: 等待窗口 (监控进程)
        self.update_progress(LaunchStage::WaitWindow, 0.0, "等待游戏加载...")
            .await;
        // 监控已在launch_process中启动

        // 完成
        self.update_progress(LaunchStage::Finished, 1.0, "启动完成")
            .await;

        Ok(result)
    }
}
