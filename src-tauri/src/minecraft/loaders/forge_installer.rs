//! 加载器安装器（Forge / NeoForge）执行模块
//!
//! 使用 bangbang93 注入器（与第三方启动器同一份 jar），库下载失败时由启动器预先补齐。

use crate::resources;
use crate::utils::cache;
use crate::{log_debug, log_error, log_info, log_warn};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 安装器超时（秒）：正常 2~5 分钟，15 分钟视为卡死
const INSTALLER_TIMEOUT_SECS: u64 = 15 * 60;

/// 保留的输出行数上限（避免安装器刷屏吃内存）
const MAX_CAPTURED_LINES: usize = 500;

/// 逐行回调（stdout 进度转发用）
type LineHandler = Box<dyn Fn(&str) + Send + 'static>;

/// 安装进度回调（0.0 ~ 1.0，来自安装器自身的输出）；用 Arc 便于两条安装路径共用
pub type ProgressHandler = Arc<dyn Fn(f64) + Send + Sync>;

/// Forge 安装进度事件
#[derive(Debug, Clone, serde::Serialize)]
pub struct ForgeInstallProgress {
    pub stage: String,
    pub progress: f64,
}

/// 释放嵌入的资源文件到缓存目录
pub fn extract_embedded_resources() -> anyhow::Result<(String, String)> {
    let cache_dir = cache::ensure_dir("forge_installer")?;

    let installer_path = cache_dir.join("forge-installer.jar");
    let wrapper_path = cache_dir.join("java-wrapper.jar");

    resources::extract_resource("forge-installer.jar", &installer_path)?;
    resources::extract_resource("java-wrapper.jar", &wrapper_path)?;

    Ok((
        installer_path.to_string_lossy().to_string(),
        wrapper_path.to_string_lossy().to_string(),
    ))
}

/// classpath 分隔符（Windows 用 `;`，其余平台用 `:`）
pub(super) fn classpath_separator() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}

/// 解码一行输出
///
/// 中文 Windows 上 JVM 的报错信息是 GBK（本地代码页），直接按 UTF-8 读会失败；
/// 先试 UTF-8，失败再按 GBK 解码，避免把安装器的错误信息整段丢掉。
pub(super) fn decode_output_line(bytes: &[u8]) -> String {
    let text = std::str::from_utf8(bytes)
        .map(str::to_string)
        .unwrap_or_else(|_| {
            let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
            decoded.into_owned()
        });
    text.trim_end_matches(['\r', '\n']).to_string()
}

/// 构建注入器启动参数
///
/// `use_wrapper` 为真时（游戏目录含中文或空格）以 java-wrapper 为主 Jar 启动，
/// 绕开 JVM 在非 ASCII 路径下找不到主类的问题。
pub(super) fn build_installer_args(
    java_major: u32,
    classpath: &str,
    wrapper_path: &str,
    mc_dir: &str,
    tmp_dir: &str,
    use_wrapper: bool,
) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    if java_major >= 9 {
        args.push("--add-exports".to_string());
        args.push("cpw.mods.bootstraplauncher/cpw.mods.bootstraplauncher=ALL-UNNAMED".to_string());
    }

    if use_wrapper {
        // 系统属性必须写成 -Dkey=value 单个参数，
        // 拆成两个参数会被 JVM 当成主类名，安装器根本不会启动。
        args.push(format!("-Doolloo.jlw.tmpdir={}", tmp_dir));
        args.push("-cp".to_string());
        args.push(classpath.to_string());
        args.push("-jar".to_string());
        args.push(wrapper_path.to_string());
        args.push("com.bangbang93.ForgeInstaller".to_string());
        args.push(mc_dir.to_string());
    } else {
        args.push("-cp".to_string());
        args.push(classpath.to_string());
        args.push("com.bangbang93.ForgeInstaller".to_string());
        args.push(mc_dir.to_string());
    }

    args
}

/// 在独立线程里持续读取输出流，按行写入共享缓冲
fn spawn_output_reader<R: std::io::Read + Send + 'static>(
    reader: R,
    sink: Arc<Mutex<Vec<String>>>,
    tag: &'static str,
    on_line: Option<LineHandler>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buffered = BufReader::new(reader);
        let mut raw: Vec<u8> = Vec::new();
        loop {
            raw.clear();
            match buffered.read_until(b'\n', &mut raw) {
                Ok(0) => break,
                Ok(_) => {
                    let line = decode_output_line(&raw);
                    log_debug!("[installer {}] {}", tag, line);
                    if let Some(ref handler) = on_line {
                        handler(&line);
                    }
                    if let Ok(mut lines) = sink.lock() {
                        lines.push(line);
                        if lines.len() > MAX_CAPTURED_LINES {
                            lines.remove(0);
                        }
                    }
                }
                Err(e) => {
                    log_warn!("安装器 {} 输出读取异常: {}", tag, e);
                    break;
                }
            }
        }
    })
}

/// 等待进程退出，超时则强杀
fn wait_with_timeout(child: &mut Child, timeout_secs: u64) -> anyhow::Result<ExitStatus> {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(anyhow::anyhow!(
                "安装器超过 {} 分钟未完成，已终止",
                timeout_secs / 60
            ));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// 把安装器完整输出落盘，便于用户回传排查（失败不影响安装流程）
fn write_installer_log(
    tag: &str,
    java_path: &str,
    args: &[String],
    lines: &[String],
) -> Option<String> {
    let cache_dir = cache::ensure_dir("forge_installer").ok()?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let log_path = cache_dir.join(format!("installer-{}-{}.log", tag, stamp));
    let mut content = String::new();
    content.push_str(&format!(
        "java: {}\nargs: {}\n\n",
        java_path,
        args.join(" ")
    ));
    for line in lines {
        content.push_str(line);
        content.push('\n');
    }
    match std::fs::write(&log_path, content) {
        Ok(()) => Some(log_path.to_string_lossy().to_string()),
        Err(e) => {
            log_warn!("写入安装器日志失败: {}", e);
            None
        }
    }
}

/// 执行安装器进程并收集输出（两条安装路径共用）
///
/// `on_progress` 接收安装器输出的 0.0~1.0 进度；`working_dir` 用于官方安装器定位游戏目录。
fn run_installer_process(
    java_path: &str,
    args: &[String],
    working_dir: Option<&Path>,
    log_tag: &str,
    progress: Option<ProgressHandler>,
) -> anyhow::Result<()> {
    let mut command = Command::new(java_path);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }

    let mut child = command
        .spawn()
        .map_err(|e| anyhow::anyhow!("无法启动 Java 安装器（{}）：{}", java_path, e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("无法读取安装器输出"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("无法读取安装器错误输出"))?;

    let sink: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let progress_handler: Option<LineHandler> = progress.map(|handler| {
        let boxed: LineHandler = Box::new(move |line: &str| {
            if let Some(value) = parse_progress_line(line) {
                handler(value);
            }
        });
        boxed
    });

    let stdout_handle = spawn_output_reader(stdout, Arc::clone(&sink), "stdout", progress_handler);
    let stderr_handle = spawn_output_reader(stderr, Arc::clone(&sink), "stderr", None);

    let status = wait_with_timeout(&mut child, INSTALLER_TIMEOUT_SECS);

    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    let lines: Vec<String> = sink.lock().map(|guard| guard.clone()).unwrap_or_default();
    let log_path = write_installer_log(log_tag, java_path, args, &lines);

    let status = status?;

    let has_success_marker = lines.iter().rev().take(20).any(|l| l.trim() == "true");
    if status.success() {
        if !has_success_marker {
            log_info!("[{}] 安装器退出码为 0", log_tag);
        }
        return Ok(());
    }
    if has_success_marker {
        log_warn!("[{}] 安装器退出码异常但输出了成功标记，按成功处理", log_tag);
        return Ok(());
    }

    log_error!("[{}] Installation failed. Full output:", log_tag);
    for (index, line) in lines.iter().enumerate() {
        log_error!("[{}]   {}: {}", log_tag, index, line);
    }

    let exit_code = status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "未知".to_string());
    let tail_start = lines.len().saturating_sub(10);
    let tail: Vec<&str> = lines[tail_start..].iter().map(String::as_str).collect();
    let mut message = if tail.is_empty() {
        format!(
            "安装器执行失败（退出码 {}），且没有任何输出；常见原因是 Java 无法启动或路径异常",
            exit_code
        )
    } else {
        format!(
            "安装器执行失败（退出码 {}），最近输出：{}",
            exit_code,
            tail.join(" | ")
        )
    };
    if lines
        .iter()
        .any(|line| line.contains("Invalid number of redirects"))
    {
        message
            .push_str("。原因是下载镜像的重定向次数超过安装器上限，可尝试在设置里切换下载源后重试");
    }
    if let Some(path) = log_path {
        message.push_str(&format!("。完整日志：{}", path));
    }
    Err(anyhow::anyhow!(message))
}

/// 用注入器安装（镜像加速路径）
pub fn run_forge_installer(
    java_path: &str,
    installer_path: &str,
    injector_path: &str,
    wrapper_path: &str,
    mc_dir: &str,
    is_neoforge: bool,
    on_progress: Option<ProgressHandler>,
) -> anyhow::Result<()> {
    let loader_name = if is_neoforge { "NeoForge" } else { "Forge" };

    for (label, path) in [("官方安装器", installer_path), ("注入器", injector_path)] {
        if !Path::new(path).exists() {
            return Err(anyhow::anyhow!(
                "{} 文件缺失: {}，请重试安装（若反复失败请检查下载源）",
                label,
                path
            ));
        }
    }

    let java_major = crate::minecraft::java::detect_java_version(java_path).unwrap_or(8);
    let classpath = format!(
        "{}{}{}",
        injector_path,
        classpath_separator(),
        installer_path
    );

    let use_wrapper = mc_dir.chars().any(|c| !c.is_ascii() || c == ' ');
    let tmp_dir = Path::new(mc_dir)
        .parent()
        .unwrap_or(Path::new(mc_dir))
        .to_string_lossy()
        .to_string();

    let args = build_installer_args(
        java_major,
        &classpath,
        wrapper_path,
        mc_dir,
        &tmp_dir,
        use_wrapper,
    );

    log_info!(
        "[{}] Starting injector: {} {}",
        loader_name,
        java_path,
        args.join(" ")
    );

    run_installer_process(
        java_path,
        &args,
        None,
        &loader_name.to_lowercase(),
        on_progress,
    )
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
