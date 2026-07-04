//! Forge Installer 注入器模块
//! 使用 bangbang93 的 forge_installer.jar 进行 Forge/NeoForge 安装

use crate::{log_info, log_debug};
use crate::resources;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

/// Forge 安装进度事件
#[derive(Debug, Clone, serde::Serialize)]
pub struct ForgeInstallProgress {
    pub stage: String,
    pub progress: f64,
}

/// 释放嵌入的资源文件到缓存目录
pub fn extract_embedded_resources(cache_dir: &Path) -> anyhow::Result<(String, String)> {
    std::fs::create_dir_all(cache_dir)?;

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
    let java_major = get_java_major_version(java_path).unwrap_or(8);

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

    log_info!("[{}] Starting installer: java {}", loader_name, args.join(" "));

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

    // 读取 stdout
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

    // 读取 stderr
    let stderr_reader = BufReader::new(stderr);
    for line in stderr_reader.lines() {
        let line = line?;
        log_debug!("[{}] stderr: {}", loader_name, line);
        last_lines.push(line);
        if last_lines.len() > 100 {
            last_lines.remove(0);
        }
    }

    // 等待进程结束
    let _status = child.wait()?;

    // 检查是否成功（最后 5 行中是否有 "true"）
    let success = last_lines.iter().rev().take(5).any(|l| l.trim() == "true");

    if success {
        log_info!("[{}] Installation successful", loader_name);
        Ok(())
    } else {
        let last_lines_str: Vec<&str> = last_lines.iter().rev().take(5).map(|s| s.as_str()).collect();
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

/// 获取 Java 主版本号
fn get_java_major_version(java_path: &str) -> Option<u32> {
    let output = Command::new(java_path)
        .arg("-version")
        .output()
        .ok()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let re = regex::Regex::new(r#"version "(\d+)\."#).ok()?;
    let captures = re.captures(&stderr)?;
    captures[1].parse().ok()
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
