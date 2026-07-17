//! Forge Installer 注入器模块
//! 使用 bangbang93 的 forge_installer.jar 进行 Forge/NeoForge 安装

use crate::resources;
use crate::{log_debug, log_error, log_info, log_warn};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

/// Forge 安装进度事件
#[derive(Debug, Clone, serde::Serialize)]
pub struct ForgeInstallProgress {
    pub stage: String,
    pub progress: f64,
}

/// 获取缓存目录（参考 PCL2：使用临时目录）
fn get_cache_dir() -> std::path::PathBuf {
    std::env::temp_dir().join("MoLaunch").join("Cache")
}

/// 释放嵌入的资源文件到缓存目录
pub fn extract_embedded_resources() -> anyhow::Result<(String, String)> {
    let cache_dir = get_cache_dir();
    std::fs::create_dir_all(&cache_dir)?;

    let installer_path = cache_dir.join("forge-installer.jar");
    let wrapper_path = cache_dir.join("java-wrapper.jar");

    resources::extract_resource("forge-installer.jar", &installer_path)?;
    resources::extract_resource("java-wrapper.jar", &wrapper_path)?;

    Ok((
        installer_path.to_string_lossy().to_string(),
        wrapper_path.to_string_lossy().to_string(),
    ))
}

/// 运行 Forge 安装器
///
/// # Arguments
/// * `java_path` - Java 可执行文件路径
/// * `installer_path` - 官方 Forge installer.jar 路径
/// * `injector_path` - bangbang93 的 forge_installer.jar 路径
/// * `wrapper_path` - java-wrapper.jar 路径
/// * `mc_dir` - Minecraft 游戏目录
/// * `is_neoforge` - 是否为 NeoForge
/// * `progress_callback` - 进度回调
pub fn run_forge_installer(
    java_path: &str,
    installer_path: &str,
    injector_path: &str,
    wrapper_path: &str,
    mc_dir: &str,
    is_neoforge: bool,
    progress_callback: Option<Box<dyn Fn(ForgeInstallProgress) + Send>>,
) -> anyhow::Result<()> {
    let loader_name = if is_neoforge { "NeoForge" } else { "Forge" };

    // 检测 Java 版本
    let java_major = crate::minecraft::java::detect_java_version(java_path).unwrap_or(8);

    // 构建 classpath
    let classpath = format!("{};{}", injector_path, installer_path);

    // 构建参数
    let mut args: Vec<String> = Vec::new();

    // Java 9+ 需要添加 --add-exports
    if java_major >= 9 {
        args.push("--add-exports".to_string());
        args.push("cpw.mods.bootstraplauncher/cpw.mods.bootstraplauncher=ALL-UNNAMED".to_string());
    }

    // 使用 JavaWrapper 绕过路径问题
    let use_wrapper = mc_dir.chars().any(|c| !c.is_ascii() || c == ' ');

    if use_wrapper {
        // 以 JavaWrapper 为主 Jar
        args.push("-Doolloo.jlw.tmpdir".to_string());
        let tmp_dir = Path::new(mc_dir).parent().unwrap_or(Path::new(mc_dir));
        args.push(tmp_dir.to_string_lossy().to_string());

        args.push("-cp".to_string());
        args.push(classpath.clone());

        args.push("-jar".to_string());
        args.push(wrapper_path.to_string());

        args.push("com.bangbang93.ForgeInstaller".to_string());
        args.push(mc_dir.to_string());
    } else {
        args.push("-cp".to_string());
        args.push(classpath);

        args.push("com.bangbang93.ForgeInstaller".to_string());
        args.push(mc_dir.to_string());
    }

    log_info!(
        "[{}] Starting installer: {} {}",
        loader_name,
        java_path,
        args.join(" ")
    );

    // 启动进程
    let mut child = Command::new(java_path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // 读取输出
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut last_lines: Vec<String> = Vec::new();

    // 在独立线程读取 stderr，避免 stdout/stderr 顺序读取时管道缓冲区填满死锁
    // （Java 安装器可能向 stderr 大量输出，同时 stdout 也在产生数据）
    let stderr_loader_name = loader_name.to_string();
    let stderr_handle = std::thread::spawn(move || {
        let stderr_reader = BufReader::new(stderr);
        let mut lines: Vec<String> = Vec::new();
        for line in stderr_reader.lines() {
            match line {
                Ok(l) => {
                    log_debug!("[{}] stderr: {}", stderr_loader_name, l);
                    lines.push(l);
                    if lines.len() > 100 {
                        lines.remove(0);
                    }
                }
                Err(e) => {
                    log_warn!("[{}] stderr 读取异常: {}", stderr_loader_name, e);
                    break;
                }
            }
        }
        lines
    });

    // 主线程读 stdout
    let stdout_reader = BufReader::new(stdout);
    for line in stdout_reader.lines() {
        let line = line?;
        log_debug!("[{}] stdout: {}", loader_name, line);

        // 解析进度
        if let Some(ref callback) = progress_callback {
            let progress = parse_progress_line(&line);
            if let Some(prog) = progress {
                callback(ForgeInstallProgress {
                    stage: line.clone(),
                    progress: prog,
                });
            }
        }

        last_lines.push(line.clone());
        if last_lines.len() > 100 {
            last_lines.remove(0);
        }
    }

    // 等待 stderr 线程结束，合并行
    let stderr_lines = stderr_handle
        .join()
        .map_err(|e| anyhow::anyhow!("stderr 读取线程崩溃: {:?}", e))?;
    last_lines.extend(stderr_lines);

    // 参考 PCL2：等待进程完全退出
    // PCL2: Do Until process.HasExited + Thread.Sleep(10)
    // Rust: child.wait() 会阻塞直到进程退出
    let _status = child.wait()?;

    // 参考 PCL2：等待 I/O 流完全关闭
    // PCL2: outputWaitHandle.WaitOne(10000) + errorWaitHandle.WaitOne(10000)
    // 我们已经通过读取 stdout/stderr 循环等待流关闭了
    // 但 Java 运行时可能还有文件句柄未释放，等待一下
    std::thread::sleep(std::time::Duration::from_secs(1));

    // 检查是否成功（最后 5 行中是否有 "true"）
    let success = last_lines.iter().rev().take(5).any(|l| l.trim() == "true");

    if success {
        log_info!("[{}] Installation successful", loader_name);
        Ok(())
    } else {
        // 输出所有日志以便诊断
        log_error!("[{}] Installation failed. Full output:", loader_name);
        for (i, line) in last_lines.iter().enumerate() {
            log_error!("[{}]   {}: {}", loader_name, i, line);
        }
        let last_lines_str: Vec<&str> = last_lines
            .iter()
            .rev()
            .take(10)
            .map(|s| s.as_str())
            .collect();
        Err(anyhow::anyhow!(
            "{} installer failed, last lines: {:?}",
            loader_name,
            last_lines_str
        ))
    }
}

/// 解析进度行
fn parse_progress_line(line: &str) -> Option<f64> {
    match line.trim() {
        "Extracting json" => Some(0.07),
        "Downloading libraries" => Some(0.08),
        "File exists: Checksum validated." => Some(0.11),
        "Building Processors" => Some(0.18),
        "Task: DOWNLOAD_MOJMAPS" => Some(0.20),
        "Task: MERGE_MAPPING" => Some(0.30),
        "Splitting: " => Some(0.35),
        "Parameter Annotations" => Some(0.40),
        "Processing Complete" => Some(0.50),
        "log: null" => Some(0.50),
        "Sorting" => Some(0.65),
        "Remapping final jar" => Some(0.72),
        "Remapping jar... 50%" => Some(0.76),
        "Remapping jar... 100%" => Some(0.81),
        "Injecting profile" => Some(0.91),
        _ => None,
    }
}

/// 检测是否需要使用注入器（新版 Forge >= 20 或 NeoForge）
pub fn needs_injector(loader_version: &str, is_neoforge: bool) -> bool {
    if is_neoforge {
        return true;
    }

    // 检查 Forge 主版本号 >= 20
    let major: u32 = loader_version
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    major >= 20
}
