//! Natives 原生库解压

use std::path::PathBuf;

use crate::{log_info, log_warn};

use super::{LaunchError, LaunchPipeline, LaunchStage};

impl LaunchPipeline {
    /// 解压Natives
    pub(super) async fn extract_natives(&self) -> Result<(), LaunchError> {
        let version_dir = self
            .config
            .game_dir
            .join("versions")
            .join(&self.config.version_id);
        let natives_dir = version_dir.join(format!("{}-natives", self.config.version_id));

        // 创建natives目录
        tokio::fs::create_dir_all(&natives_dir)
            .await
            .map_err(|e| LaunchError {
                stage: LaunchStage::ExtractNatives,
                message: format!("创建natives目录失败: {}", e),
                is_user_facing: false,
            })?;

        // 读取版本JSON
        let json_path = version_dir.join(format!("{}.json", self.config.version_id));
        let json_content =
            tokio::fs::read_to_string(&json_path)
                .await
                .map_err(|e| LaunchError {
                    stage: LaunchStage::ExtractNatives,
                    message: format!("读取版本JSON失败: {}", e),
                    is_user_facing: false,
                })?;

        let json: serde_json::Value =
            serde_json::from_str(&json_content).map_err(|e| LaunchError {
                stage: LaunchStage::ExtractNatives,
                message: format!("解析版本JSON失败: {}", e),
                is_user_facing: false,
            })?;

        // 查找natives库
        if let Some(libraries) = json["libraries"].as_array() {
            let total = libraries.len();
            for (i, lib) in libraries.iter().enumerate() {
                // 应用 rules 过滤（平台适配）
                let rules: Option<Vec<serde_json::Value>> = lib
                    .get("rules")
                    .and_then(|v| v.as_array())
                    .map(|a| a.clone());
                if !crate::minecraft::version::libraries::check_rules(&rules) {
                    continue;
                }

                // 模式 A（旧版）：library 有 "natives" 字段 + "downloads.classifiers"
                if let Some(natives_field) = lib.get("natives").and_then(|v| v.as_object()) {
                    let platform_key = if cfg!(target_os = "windows") {
                        "windows"
                    } else if cfg!(target_os = "macos") {
                        "osx"
                    } else {
                        "linux"
                    };

                    let classifier_key = match natives_field.get(platform_key).and_then(|v| v.as_str()) {
                        Some(c) => c.to_string(),
                        None => continue,
                    };

                    if let Some(classifiers) = lib["downloads"]["classifiers"].as_object() {
                        let artifact = classifiers.get(&classifier_key).or_else(|| {
                            let base = classifier_key
                                .split('-')
                                .take(2)
                                .collect::<Vec<_>>()
                                .join("-");
                            if base != classifier_key {
                                classifiers.get(&base)
                            } else {
                                None
                            }
                        });

                        if let Some(artifact) = artifact {
                            if let Some(path) = artifact["path"].as_str() {
                                let jar_path = self.config.game_dir.join("libraries").join(path);
                                if jar_path.exists() {
                                    let jar_sha1 = artifact["sha1"].as_str();
                                    log_info!(
                                        "[Natives] Processing native JAR: {} (expected sha1: {:?})",
                                        jar_path.display(),
                                        jar_sha1
                                    );
                                    self.extract_native_jar(&jar_path, &natives_dir, jar_sha1)
                                        .await?;
                                }
                            }
                        }
                    }
                    self.update_progress(
                        LaunchStage::ExtractNatives,
                        (i + 1) as f64 / total as f64,
                        "正在解压原生库...",
                    )
                    .await;
                    continue;
                }

                // 模式 B（Forge 26.2+ 新格式）：library 无 "natives" 字段，但 name 含 classifier（如 "natives-windows-x86"）
                // 这类直接用 downloads.artifact.path 解压
                if let Some(name) = lib["name"].as_str() {
                    let parts: Vec<&str> = name.split(':').collect();
                    if parts.len() > 3 {
                        let classifier = parts[3];
                        if classifier.starts_with("natives-") {
                            // 架构过滤：避免解压错误架构的 native
                            if !crate::minecraft::version::libraries::is_native_matching_arch(
                                classifier,
                            ) {
                                self.update_progress(
                                    LaunchStage::ExtractNatives,
                                    (i + 1) as f64 / total as f64,
                                    "正在解压原生库...",
                                )
                                .await;
                                continue;
                            }
                            if let Some(path) = lib["downloads"]["artifact"]["path"].as_str() {
                                let jar_path = self.config.game_dir.join("libraries").join(path);
                                if jar_path.exists() {
                                    let jar_sha1 = lib["downloads"]["artifact"]["sha1"].as_str();
                                    log_info!(
                                        "[Natives] Processing native JAR: {} (expected sha1: {:?})",
                                        jar_path.display(),
                                        jar_sha1
                                    );
                                    self.extract_native_jar(&jar_path, &natives_dir, jar_sha1)
                                        .await?;
                                }
                            }
                        }
                    }
                }

                self.update_progress(
                    LaunchStage::ExtractNatives,
                    (i + 1) as f64 / total as f64,
                    "正在解压原生库...",
                )
                .await;
            }
        }

        Ok(())
    }

    /// 解压单个native jar
    ///
    /// `expected_sha1` 为版本 JSON 中记录的 JAR 文件 SHA1（可选）。
    /// - 若提供：先校验 JAR 文件 SHA1，匹配才解压；不匹配则跳过提取并记录警告。
    /// - 若为 None：记录警告（无法校验），仍按原逻辑解压。
    /// 每个提取出的 DLL/SO/DYLIB 会计算并记录其 SHA1，便于审计。
    async fn extract_native_jar(
        &self,
        jar_path: &PathBuf,
        natives_dir: &PathBuf,
        expected_sha1: Option<&str>,
    ) -> Result<(), LaunchError> {
        let jar_path = jar_path.clone();
        let natives_dir = natives_dir.clone();
        let expected_sha1 = expected_sha1.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            use std::fs::File;
            use std::io::Read;

            // SHA1 校验：如果提供了预期 SHA1，先校验 JAR 文件完整性（CWE-494/CWE-345）
            if let Some(ref expected) = expected_sha1 {
                let jar_bytes = match std::fs::read(&jar_path) {
                    Ok(b) => b,
                    Err(e) => return Err(format!("读取jar文件失败: {}", e)),
                };
                let actual = crate::minecraft::utils::file_checker::compute_sha1_hex(&jar_bytes);
                if actual.eq_ignore_ascii_case(expected) {
                    log_info!(
                        "[Natives] JAR SHA1 verified: {} (sha1={})",
                        jar_path.display(),
                        actual
                    );
                } else {
                    log_warn!(
                        "[Natives] JAR SHA1 mismatch for {}: expected={}, actual={} — skipping extraction",
                        jar_path.display(),
                        expected,
                        actual
                    );
                    return Ok(());
                }
            } else {
                log_warn!(
                    "[Natives] No expected SHA1 for JAR {}, skipping verification",
                    jar_path.display()
                );
            }

            log_info!(
                "[Natives] Extracting native JAR: {}",
                jar_path.display()
            );

            let file = File::open(&jar_path).map_err(|e| format!("打开jar失败: {}", e))?;
            let mut archive =
                zip::ZipArchive::new(file).map_err(|e| format!("读取zip失败: {}", e))?;

            for i in 0..archive.len() {
                let mut entry = archive
                    .by_index(i)
                    .map_err(|e| format!("读取zip条目失败: {}", e))?;

                let entry_name = entry.name().to_string();

                // 只提取dll/so/dylib文件
                if entry_name.ends_with(".dll")
                    || entry_name.ends_with(".so")
                    || entry_name.ends_with(".dylib")
                {
                    let out_path =
                        natives_dir.join(std::path::Path::new(&entry_name).file_name().unwrap());

                    let mut buffer = Vec::new();
                    entry
                        .read_to_end(&mut buffer)
                        .map_err(|e| format!("读取文件失败: {}", e))?;

                    // 计算提取文件的 SHA1 用于审计日志
                    let file_sha1 = crate::minecraft::utils::file_checker::compute_sha1_hex(&buffer);

                    std::fs::write(&out_path, &buffer)
                        .map_err(|e| format!("写入文件失败: {}", e))?;

                    log_info!(
                        "[Natives] Extracted: {} (size={}, sha1={})",
                        out_path.display(),
                        buffer.len(),
                        file_sha1
                    );
                }
            }

            Ok(())
        })
        .await
        .map_err(|e| LaunchError {
            stage: LaunchStage::ExtractNatives,
            message: format!("任务执行失败: {}", e),
            is_user_facing: false,
        })?
        .map_err(|e: String| LaunchError {
            stage: LaunchStage::ExtractNatives,
            message: e,
            is_user_facing: false,
        })
    }
}
