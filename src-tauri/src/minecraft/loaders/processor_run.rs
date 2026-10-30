//! 执行 install_profile 里声明的 processors
//!
//! 每个 processor 是一次 `java -cp <classpath> <MainClass> <args...>`，Main-Class 从
//! processor jar 的 MANIFEST 读，不能写死。产物已就绪的步骤会跳过——重装时
//! 不必把几分钟的重写流程再跑一遍。

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde_json::Value;

use super::processor_args::{resolve_arguments, ProcessorContext};

/// 单个 processor 的超时。改写整个 jar 也就几分钟，超过这个数基本是卡死了
const PROCESSOR_TIMEOUT_SECS: u64 = 600;

/// 报错时回看的日志行数
const LOG_TAIL_LINES: usize = 10;

/// 一个待执行的 processor
pub(crate) struct PlannedProcessor {
    /// 日志里显示的名字（jar 坐标）
    pub label: String,
    /// processor 自己的 jar，Main-Class 从这里读
    pub jar: PathBuf,
    /// 执行时要挂上的 classpath（不含 jar 本身）
    pub classpath: Vec<PathBuf>,
    /// 已解析占位符的参数
    pub arguments: Vec<String>,
    /// 声明的产物；全都存在就跳过这一步
    pub outputs: Vec<PathBuf>,
}

/// 从参数里挑出产物路径
///
/// `--output` 后面只跟一个；`--outputs` 后面可以跟一串，直到下一个选项为止。
/// 早先只取紧跟的第一个，导致多输出步骤被误判成「产物齐了」而整步跳过。
pub(crate) fn outputs_of(arguments: &[String]) -> Vec<PathBuf> {
    let mut outputs = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--output" => {
                if let Some(value) = arguments.get(index + 1) {
                    outputs.push(PathBuf::from(value));
                }
                index += 2;
            }
            "--outputs" => {
                index += 1;
                while index < arguments.len() && !arguments[index].starts_with("--") {
                    outputs.push(PathBuf::from(&arguments[index]));
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }
    outputs
}

fn coordinate_to_path(coordinate: &str, game_dir: &Path) -> PathBuf {
    super::shared::maven_path_to_local(coordinate, game_dir)
}

/// 按 install_profile 排出 client 侧要跑的 processor
///
/// 缺 processor jar 直接报错：那说明依赖基础就不完整，跳过只会让安装
/// 「看起来成功」却少跑一步。
pub(crate) fn plan_client_processors(
    profile: &Value,
    game_dir: &Path,
    version: &str,
) -> anyhow::Result<Vec<PlannedProcessor>> {
    let data = profile_data(profile);
    let minecraft_jar = game_dir
        .join("versions")
        .join(version)
        .join(format!("{}.jar", version));
    let context = ProcessorContext {
        game_dir,
        minecraft_jar: &minecraft_jar,
        installer: Path::new(""),
        binpatch: None,
        version,
        data: &data,
    };
    let Some(processors) = profile.get("processors").and_then(|value| value.as_array()) else {
        return Ok(Vec::new());
    };

    let mut planned = Vec::new();
    for processor in processors {
        let runs_on_client = processor
            .get("sides")
            .and_then(|value| value.as_array())
            .map(|sides| sides.iter().any(|side| side.as_str() == Some("client")))
            .unwrap_or(true);
        if !runs_on_client {
            continue;
        }
        let Some(jar_coordinate) = processor.get("jar").and_then(|value| value.as_str()) else {
            return Err(anyhow::anyhow!(
                "install_profile 里有个 processor 没声明 jar，无法执行"
            ));
        };
        let jar = coordinate_to_path(jar_coordinate, game_dir);
        if !jar.is_file() {
            return Err(anyhow::anyhow!(
                "缺少 processor jar：{}（{}）。安装器需要的库没下全，请重试安装。",
                jar_coordinate,
                jar.display()
            ));
        }
        let classpath: Vec<PathBuf> = processor
            .get("classpath")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .map(|coordinate| coordinate_to_path(coordinate, game_dir))
                    .collect()
            })
            .unwrap_or_default();
        let arguments: Vec<String> = processor
            .get("args")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let resolved = resolve_arguments(&arguments, &context);
        let outputs = outputs_of(&resolved);
        planned.push(PlannedProcessor {
            label: jar_coordinate.to_string(),
            jar,
            classpath,
            arguments: resolved,
            outputs,
        });
    }
    Ok(planned)
}

/// install_profile 的 data 表，按 client 侧取值
fn profile_data(profile: &Value) -> std::collections::HashMap<String, String> {
    let mut data = std::collections::HashMap::new();
    let Some(table) = profile.get("data").and_then(|value| value.as_object()) else {
        return data;
    };
    for (key, value) in table {
        let picked = value
            .as_object()
            .and_then(|object| object.get("client"))
            .and_then(|client| client.as_str())
            .or_else(|| {
                value
                    .as_array()
                    .and_then(|array| array.first())
                    .and_then(|first| first.as_str())
            })
            .or_else(|| value.as_str());
        if let Some(picked) = picked {
            data.insert(key.clone(), picked.to_string());
        }
    }
    data
}

/// 读 processor jar 的 Main-Class
fn read_main_class(jar_path: &Path) -> anyhow::Result<String> {
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut manifest = String::new();
    {
        let mut entry = archive.by_name("META-INF/MANIFEST.MF")?;
        std::io::Read::read_to_string(&mut entry, &mut manifest)?;
    }
    for line in manifest.lines() {
        if let Some(rest) = line.strip_prefix("Main-Class:") {
            let trimmed = rest.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }
    Err(anyhow::anyhow!(
        "{} 的 MANIFEST 里没有 Main-Class",
        jar_path.display()
    ))
}

/// 等子进程结束，超时就杀掉
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> anyhow::Result<std::process::ExitStatus> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if started.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(anyhow::anyhow!(
                "超过 {} 秒仍未结束，已强制终止",
                timeout.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// 读日志文件末尾若干行
fn log_tail(path: &Path) -> String {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<&str> = content.lines().rev().take(LOG_TAIL_LINES).collect();
    lines.reverse();
    lines.join("\n")
}

/// 依次执行，产物齐的步骤跳过；返回实际执行的步数
///
/// `on_progress` 收到「已完成步数 / 总步数」，让上层能把进度条往前推：
/// 几分钟的重写过程不该让界面看起来像卡死了。
pub(crate) fn run_planned_processors(
    java_path: &str,
    planned: &[PlannedProcessor],
    on_progress: Option<&dyn Fn(usize, usize)>,
) -> anyhow::Result<usize> {
    let total = planned.len();
    let mut executed = 0;
    let log_dir = crate::storage::cache::ensure_dir("processor_logs")?;

    for (index, step) in planned.iter().enumerate() {
        if !step.outputs.is_empty() && step.outputs.iter().all(|path| path.is_file()) {
            crate::log_debug!("[Processor] {} 产物已在，跳过", step.label);
            if let Some(callback) = on_progress {
                callback(index + 1, total);
            }
            continue;
        }
        let main_class = read_main_class(&step.jar)?;
        let mut classpath_entries = step.classpath.clone();
        classpath_entries.push(step.jar.clone());
        let classpath = std::env::join_paths(&classpath_entries)
            .map_err(|e| anyhow::anyhow!("classpath 拼接失败: {}", e))?;

        let log_path = log_dir.join(format!("processor-{}-{}.log", index, executed));
        let log_file = std::fs::File::create(&log_path)?;
        let log_clone = log_file.try_clone()?;

        crate::log_info!(
            "[Processor] 执行第 {}/{} 步 {}（日志 {}）",
            index + 1,
            total,
            step.label,
            log_path.display()
        );
        let mut child = std::process::Command::new(java_path)
            .arg("-cp")
            .arg(&classpath)
            .arg(&main_class)
            .args(&step.arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(log_clone))
            .spawn()?;
        let status = wait_with_timeout(&mut child, Duration::from_secs(PROCESSOR_TIMEOUT_SECS))
            .map_err(|e| anyhow::anyhow!("processor {} {}", step.label, e))?;
        if !status.success() {
            return Err(anyhow::anyhow!(
                "processor {} 执行失败（退出码 {:?}），日志末尾：\n{}\n完整日志：{}",
                step.label,
                status.code(),
                log_tail(&log_path),
                log_path.display()
            ));
        }
        executed += 1;
        if let Some(callback) = on_progress {
            callback(index + 1, total);
        }
    }
    crate::log_info!("[Processor] 实际执行 {} 个 processor", executed);
    Ok(executed)
}

/// 读安装器里的 profile 并跑一遍 client 侧 processors
pub(crate) fn run_from_installer(
    java_path: &str,
    installer_path: &Path,
    mc_version: &str,
    game_dir: &Path,
    on_progress: Option<&dyn Fn(usize, usize)>,
) -> anyhow::Result<usize> {
    let file = std::fs::File::open(installer_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut content = String::new();
    {
        let mut entry = archive.by_name("install_profile.json")?;
        std::io::Read::read_to_string(&mut entry, &mut content)?;
    }
    let profile: Value = serde_json::from_str(&content)?;
    let planned = plan_client_processors(&profile, game_dir, mc_version)?;
    run_planned_processors(java_path, &planned, on_progress)
}

#[cfg(test)]
#[path = "processor_run_test.rs"]
mod processor_run_test;
