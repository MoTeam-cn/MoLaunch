//! 执行 install_profile 里声明的 processors
//!
//! 每个 processor 是一次 `java -cp <classpath> <MainClass> <args...>`，Main-Class 从
//! processor jar 的 MANIFEST 读，不能写死。产物已就绪的步骤会跳过——重装时
//! 不必把几分钟的重写流程再跑一遍。

use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde_json::Value;

use super::processor_args::{resolve_arguments, ProcessorContext};

/// processor 输出里回看的行数，用于报错
const STDERR_TAIL_LINES: usize = 8;

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

/// 从参数里挑出 `--output` / `--outputs` 声明的产物路径
pub(crate) fn outputs_of(arguments: &[String]) -> Vec<PathBuf> {
    let mut outputs = Vec::new();
    let mut index = 0;
    while index + 1 < arguments.len() {
        let argument = arguments[index].as_str();
        if argument == "--output" || argument == "--outputs" {
            outputs.push(PathBuf::from(&arguments[index + 1]));
            index += 2;
            continue;
        }
        index += 1;
    }
    outputs
}

fn coordinate_to_path(coordinate: &str, game_dir: &Path) -> PathBuf {
    super::shared::maven_path_to_local(coordinate, game_dir)
}

/// 按 install_profile 排出 client 侧要跑的 processor
pub(crate) fn plan_client_processors(
    profile: &Value,
    game_dir: &Path,
    version: &str,
) -> Vec<PlannedProcessor> {
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
        return Vec::new();
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
            continue;
        };
        let jar = coordinate_to_path(jar_coordinate, game_dir);
        if !jar.is_file() {
            crate::log_warn!(
                "[Processor] 缺少 processor jar: {}，跳过这一步",
                jar.display()
            );
            continue;
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
    planned
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

/// 依次执行，产物齐的步骤跳过；返回实际执行的步数
pub(crate) fn run_planned_processors(
    java_path: &str,
    planned: &[PlannedProcessor],
) -> anyhow::Result<usize> {
    let mut executed = 0;
    for step in planned {
        if !step.outputs.is_empty() && step.outputs.iter().all(|path| path.is_file()) {
            crate::log_debug!("[Processor] {} 产物已在，跳过", step.label);
            continue;
        }
        let main_class = read_main_class(&step.jar)?;
        let mut classpath_entries = step.classpath.clone();
        classpath_entries.push(step.jar.clone());
        let classpath = std::env::join_paths(&classpath_entries)
            .map_err(|e| anyhow::anyhow!("classpath 拼接失败: {}", e))?;

        crate::log_info!(
            "[Processor] 执行 {} 个参数（{}）",
            step.arguments.len(),
            step.label
        );
        let output = std::process::Command::new(java_path)
            .arg("-cp")
            .arg(&classpath)
            .arg(&main_class)
            .args(&step.arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let mut tail: Vec<&str> = stderr.lines().rev().take(STDERR_TAIL_LINES).collect();
            tail.reverse();
            return Err(anyhow::anyhow!(
                "processor {} 执行失败（退出码 {:?}）：\n{}",
                step.label,
                output.status.code(),
                tail.join("\n")
            ));
        }
        executed += 1;
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
) -> anyhow::Result<usize> {
    let file = std::fs::File::open(installer_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut content = String::new();
    {
        let mut entry = archive.by_name("install_profile.json")?;
        std::io::Read::read_to_string(&mut entry, &mut content)?;
    }
    let profile: Value = serde_json::from_str(&content)?;
    let planned = plan_client_processors(&profile, game_dir, mc_version);
    run_planned_processors(java_path, &planned)
}

#[cfg(test)]
#[path = "processor_run_test.rs"]
mod processor_run_test;
