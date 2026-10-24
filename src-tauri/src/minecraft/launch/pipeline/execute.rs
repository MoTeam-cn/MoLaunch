//! 启动流程编排
//!
//! 实现 `LaunchPipeline::execute` 主流程与 `update_progress` 进度更新工具方法。

use crate::{log_info, log_warn};
use tauri::Emitter;

/// 判定「Java 版本不兼容」类的启动失败信号
const JAVA_FAILURE_SIGNATURES: [&str; 5] = [
    "UnsupportedClassVersionError",
    "has been compiled by a more recent version",
    "Could not create the Java Virtual Machine",
    "A Java Exception has occurred",
    "A JNI error has occurred",
];

use std::path::PathBuf;

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

        // 通过 Tauri event 推送进度快照（前端 listen "launch-progress"，替代轮询 get_launch_progress）
        // app_handle 在 build_launch_config 中注入；None 仅防御性兜底（不应发生）
        if let Some(app_handle) = self.config.app_handle.as_ref() {
            let _ = app_handle.emit("launch-progress", &*progress);
        }
    }

    /// 执行启动流程
    ///
    /// 自动换 Java 重试：首次启动因 Java 版本不兼容失败时，排除该 Java 后重新选择并再试一次
    /// （只重试一次，避免死循环）。用户手动指定的 Java（custom / folder 模式）不自动替换。
    pub async fn execute(&self) -> Result<LaunchResult, LaunchError> {
        let mut excluded: Vec<String> = Vec::new();
        let mut retried = false;

        loop {
            let mut chosen_java: Option<PathBuf> = None;
            match self.execute_once(&excluded, &mut chosen_java).await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    if retried || !is_java_version_failure(&err) || !self.allows_java_auto_retry() {
                        return Err(err);
                    }
                    let Some(java) = chosen_java else {
                        return Err(err);
                    };
                    let failed = java.to_string_lossy().to_string();
                    if excluded
                        .iter()
                        .any(|item| item.eq_ignore_ascii_case(&failed))
                    {
                        return Err(err);
                    }
                    log_warn!(
                        "[Launch] Java 版本相关失败，自动切换其它 Java 重试（已排除 {}）",
                        failed
                    );
                    excluded.push(failed.clone());
                    retried = true;
                    if let Some(app_handle) = self.config.app_handle.as_ref() {
                        let _ = app_handle.emit(
                            "launch-java-switch",
                            serde_json::json!({
                                "from": failed,
                                "message": "检测到 Java 版本不兼容，正在自动切换其它 Java 重试…"
                            }),
                        );
                    }
                }
            }
        }
    }

    /// 是否允许自动换 Java 重试（用户手动指定的 Java 不自动替换）
    fn allows_java_auto_retry(&self) -> bool {
        let mode = self
            .config
            .java_mode
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_lowercase();
        mode != "custom" && mode != "folder"
    }

    /// 执行一次完整启动流程
    ///
    /// `excluded` 中的 Java 会被跳过；`chosen_java` 回传本次实际使用的 Java，
    /// 供调用方在失败重试时排除。
    async fn execute_once(
        &self,
        excluded: &[String],
        chosen_java: &mut Option<PathBuf>,
    ) -> Result<LaunchResult, LaunchError> {
        log_info!(
            "Starting launch pipeline for version: {}",
            self.config.version_id
        );

        // 检查取消
        if self.is_cancelled() {
            return Err(LaunchError {
                stage: LaunchStage::Init,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }

        // 阶段1: 获取Java
        self.update_progress(LaunchStage::GetJava, 0.0, "正在检测Java...")
            .await;
        let java_path = self.detect_java_excluding(excluded).await?;
        *chosen_java = Some(java_path.clone());
        self.update_progress(LaunchStage::GetJava, 1.0, "Java检测完成")
            .await;

        // 检查取消
        if self.is_cancelled() {
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
        if self.is_cancelled() {
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
        if self.is_cancelled() {
            return Err(LaunchError {
                stage: LaunchStage::BuildArgs,
                message: "启动已取消".to_string(),
                is_user_facing: true,
            });
        }

        // 阶段4: 启动前命令（advance_run_cmd）
        // 高性能显卡设置也在这一阶段执行
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

/// 判定错误是否属于「换一个 Java 可能解决」的类型
///
/// 只看启动进程阶段的错误：Java 版本不兼容会在进程退出前把上述信息打进日志，
/// `launch_process` 会把最近日志附在错误消息里，因此匹配消息文本即可。
fn is_java_version_failure(err: &LaunchError) -> bool {
    if err.stage != LaunchStage::LaunchProcess {
        return false;
    }
    JAVA_FAILURE_SIGNATURES
        .iter()
        .any(|signature| err.message.contains(signature))
}
