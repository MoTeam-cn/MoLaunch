//! JAR 工作区（规格 §1）：安全解包、按相对路径镜像、按工作区重打包
//!
//! 解包时逐条校验上限并拒绝越界路径/符号链接；工作区镜像归档结构，
//! 因此重打包只需遍历目录即可还原条目名，新写入的语言文件也会自然带上。

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::ZipArchive;

/// 工作区元数据文件名（不写入输出 JAR）
pub const WORKSPACE_META: &str = ".molaunch-jar-meta.json";
/// 内部文件名前缀（元数据、续传标记），打包时跳过
const INTERNAL_PREFIX: &str = ".molaunch-";
const META_FORMAT: u32 = 1;

/// 解包上限（规格 §1 表格）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    pub archive_bytes: u64,
    pub entries: usize,
    pub entry_bytes: u64,
    pub total_bytes: u64,
    pub compression_ratio: f64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            archive_bytes: 1_200 * 1024 * 1024,
            entries: 200_000,
            entry_bytes: 256 * 1024 * 1024,
            total_bytes: 2_000 * 1024 * 1024,
            compression_ratio: 200.0,
        }
    }
}

/// 工作区元数据：解包结果摘要，供后续阶段判断是否已签名
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMeta {
    pub format: u32,
    pub signed: bool,
    pub entries: usize,
}

impl WorkspaceMeta {
    pub fn read(directory: &Path) -> Option<Self> {
        let raw = std::fs::read_to_string(directory.join(WORKSPACE_META)).ok()?;
        let meta: WorkspaceMeta = serde_json::from_str(&raw).ok()?;
        (meta.format == META_FORMAT).then_some(meta)
    }

    pub fn write(&self, directory: &Path) -> std::io::Result<()> {
        let raw = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(directory.join(WORKSPACE_META), raw)
    }
}

/// 解包结果
#[derive(Debug, Clone, Copy)]
pub struct Extraction {
    pub signed: bool,
    pub entries: usize,
}

/// 解包过程中的累计计数与上限校验
struct Guard {
    limits: Limits,
    entries: usize,
    expanded: u64,
}

impl Guard {
    fn new(limits: Limits) -> Self {
        Self {
            limits,
            entries: 0,
            expanded: 0,
        }
    }

    /// 记录一个条目；超出任一项上限即返回错误
    fn admit(&mut self, name: &str, size: u64, compressed: u64) -> Result<(), String> {
        self.entries += 1;
        if self.entries > self.limits.entries {
            return Err("the archive holds more entries than the safety limit".to_string());
        }
        if size > self.limits.entry_bytes {
            return Err(format!("archive entry is too large: {name}"));
        }
        self.expanded = self.expanded.saturating_add(size);
        if self.expanded > self.limits.total_bytes {
            return Err("the archive expands beyond the safety limit".to_string());
        }
        if compressed > 0 && size as f64 / compressed as f64 > self.limits.compression_ratio {
            return Err(format!(
                "archive entry has a suspicious compression ratio: {name}"
            ));
        }
        Ok(())
    }
}

/// 归档条目名 → 安全路径段；绝对路径、盘符、`..`、NUL 一律拒绝
fn safe_segments(raw: &str) -> Result<Vec<String>, String> {
    if raw.contains('\0') {
        return Err(format!("archive entry contains a NUL byte: {raw}"));
    }
    let unified = raw.replace('\\', "/");
    if unified.starts_with('/') {
        return Err(format!("archive entry uses an absolute path: {raw}"));
    }
    let mut segments = Vec::new();
    for part in unified.split('/') {
        match part {
            "" | "." => continue,
            ".." => return Err(format!("archive entry escapes the workspace: {raw}")),
            other => {
                let bytes = other.as_bytes();
                if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
                    return Err(format!("archive entry uses a drive prefix: {raw}"));
                }
                segments.push(other.to_string());
            }
        }
    }
    Ok(segments)
}

/// 在 `root` 内解析相对路径
fn resolve_inside(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let mut target = root.to_path_buf();
    for segment in safe_segments(relative)? {
        target.push(segment);
    }
    if target.strip_prefix(root).is_err() {
        return Err(format!("path escapes the workspace: {relative}"));
    }
    Ok(target)
}

/// 签名文件：`META-INF/` 下的 .SF/.RSA/.DSA/.EC（重打包后必然失效）
pub fn is_signature_entry(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    let Some(tail) = lowered.strip_prefix("meta-inf/") else {
        return false;
    };
    let Some((_, extension)) = tail.rsplit_once('.') else {
        return false;
    };
    matches!(extension, "sf" | "rsa" | "dsa" | "ec")
}

/// 内部文件：元数据与续传标记，不写入输出 JAR
pub fn is_internal_file(name: &str) -> bool {
    name == WORKSPACE_META || name.starts_with(INTERNAL_PREFIX)
}

/// 递归收集工作区相对路径（正斜杠、升序）
pub fn collect_files(root: &Path) -> Result<Vec<String>, String> {
    let mut collected = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let reader = std::fs::read_dir(&directory)
            .map_err(|e| format!("unable to read workspace directory {directory:?}: {e}"))?;
        for item in reader {
            let item = item.map_err(|e| format!("unable to read workspace entry: {e}"))?;
            let path = item.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|_| "workspace path prefix error".to_string())?;
                collected.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    collected.sort();
    Ok(collected)
}

/// 解包到已存在的 `workspace`；签名文件只记录不落盘
pub fn extract_archive(
    input: &Path,
    workspace: &Path,
    limits: &Limits,
) -> Result<Extraction, String> {
    let metadata =
        std::fs::metadata(input).map_err(|e| format!("unable to stat the input archive: {e}"))?;
    if !metadata.is_file() {
        return Err("the input path is not a regular file".to_string());
    }
    if metadata.len() > limits.archive_bytes {
        return Err("the archive is larger than the safety limit".to_string());
    }

    let handle = File::open(input).map_err(|e| format!("unable to open the input archive: {e}"))?;
    let mut archive =
        ZipArchive::new(handle).map_err(|e| format!("the input is not a readable JAR/ZIP: {e}"))?;

    let mut guard = Guard::new(*limits);
    let mut signed = false;

    for position in 0..archive.len() {
        let mut entry = archive
            .by_index(position)
            .map_err(|e| format!("unable to open archive entry #{position}: {e}"))?;
        let raw_name = entry.name().to_string();
        if safe_segments(&raw_name)?.is_empty() {
            continue;
        }
        if entry.is_symlink() {
            return Err(format!("archive entry is a symbolic link: {raw_name}"));
        }
        guard.admit(&raw_name, entry.size(), entry.compressed_size())?;

        if is_signature_entry(&raw_name) {
            // 签名只做记录：重打包会改变字节，保留旧签名只会让 JAR 失效
            signed = true;
            continue;
        }
        if entry.is_dir() || raw_name.ends_with('/') {
            continue;
        }

        let target = resolve_inside(workspace, &raw_name)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("unable to create a workspace directory for {raw_name}: {e}")
            })?;
        }
        let mut sink = File::create(&target)
            .map_err(|e| format!("unable to create the workspace file for {raw_name}: {e}"))?;
        std::io::copy(&mut entry, &mut sink)
            .map_err(|e| format!("unable to extract {raw_name}: {e}"))?;
    }

    WorkspaceMeta {
        format: META_FORMAT,
        signed,
        entries: guard.entries,
    }
    .write(workspace)
    .map_err(|e| format!("unable to write the workspace metadata: {e}"))?;

    Ok(Extraction {
        signed,
        entries: guard.entries,
    })
}

/// 把工作区重新打成 JAR（条目名取自相对路径，内部文件跳过）
pub fn package_archive(workspace: &Path, output: &Path) -> Result<(), String> {
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("unable to create the output directory: {e}"))?;
        }
    }
    if output.exists() {
        return Err(format!(
            "the output file already exists: {}",
            output.display()
        ));
    }

    crate::utils::fs::write_atomic_with(output, |temporary| {
        let handle =
            File::create(temporary).map_err(|e| format!("unable to create the output JAR: {e}"))?;
        let mut writer = zip::ZipWriter::new(handle);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for relative in collect_files(workspace)? {
            if is_internal_file(&relative) {
                continue;
            }
            let source = workspace.join(&relative);
            let bytes = std::fs::read(&source)
                .map_err(|e| format!("unable to read {}: {e}", source.display()))?;
            writer
                .start_file(&relative, options)
                .map_err(|e| format!("unable to write archive entry {relative}: {e}"))?;
            writer
                .write_all(&bytes)
                .map_err(|e| format!("unable to write archive entry {relative}: {e}"))?;
        }

        writer
            .finish()
            .map_err(|e| format!("unable to finalize the output JAR: {e}"))?;
        Ok(())
    })
}
