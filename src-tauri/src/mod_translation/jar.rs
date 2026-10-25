//! 模组翻译：JAR 解包与重打包
//!
//! 工作区按归档内的相对路径镜像存放，因此重打包不需要任何条目清单：
//! 解包时只写一份小体积元数据（是否含签名文件 / 条目数），打包时遍历工作区
//! 即可还原原始条目名，新生成的 zh_cn 文件也会自然带上。

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::ZipArchive;

/// 工作区元数据文件名（自身不进输出 JAR）
pub const WORKSPACE_META: &str = ".molaunch-jar-meta.json";
/// 内部文件前缀：元数据、断点续传标记都不写入输出 JAR
const INTERNAL_PREFIX: &str = ".molaunch-";
/// 历史遗留前缀，续传工作区可能仍带这些文件
const LEGACY_INTERNAL_PREFIX: &str = ".mod-translator-";

/// 解包上限，防止畸形压缩包撑爆磁盘/内存
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

/// 工作区元数据
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
        (meta.format == 1).then_some(meta)
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

/// 把归档条目名拆成安全的路径段；任何可能逃出工作区的写法一律拒绝
fn entry_segments(raw: &str) -> Result<Vec<String>, String> {
    if raw.contains('\0') {
        return Err(format!("archive entry contains a NUL byte: {raw}"));
    }
    let unified = raw.replace('\\', "/");
    if unified.starts_with('/') {
        return Err(format!("archive entry uses an absolute path: {raw}"));
    }
    let mut segments: Vec<String> = Vec::new();
    for part in unified.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            return Err(format!("archive entry escapes the workspace: {raw}"));
        }
        let bytes = part.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
            return Err(format!("archive entry uses a drive prefix: {raw}"));
        }
        segments.push(part.to_string());
    }
    Ok(segments)
}

/// 在 `root` 下解析相对路径，结果必须仍在 `root` 内
fn resolve_inside(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let segments = entry_segments(relative)?;
    let mut target = root.to_path_buf();
    for segment in &segments {
        target.push(segment);
    }
    if target.strip_prefix(root).is_err() {
        return Err(format!("path escapes the workspace: {relative}"));
    }
    Ok(target)
}

/// 签名文件：META-INF 下的 .SF/.RSA/.DSA/.EC
fn is_signature_entry(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    let Some(tail) = lowered.strip_prefix("meta-inf/") else {
        return false;
    };
    let Some((_, extension)) = tail.rsplit_once('.') else {
        return false;
    };
    matches!(extension, "sf" | "rsa" | "dsa" | "ec")
}

/// 内部文件（元数据、续传标记）：不进输出 JAR
fn is_internal_file(name: &str) -> bool {
    name == WORKSPACE_META
        || name.starts_with(INTERNAL_PREFIX)
        || name.starts_with(LEGACY_INTERNAL_PREFIX)
}

/// 递归收集工作区相对文件路径（正斜杠、已排序）
pub fn collect_files(root: &Path) -> Result<Vec<String>, String> {
    let mut collected: Vec<String> = Vec::new();
    let mut queue: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = queue.pop() {
        let reader = std::fs::read_dir(&directory)
            .map_err(|e| format!("unable to read workspace directory {directory:?}: {e}"))?;
        for item in reader {
            let item = item.map_err(|e| format!("unable to read workspace entry: {e}"))?;
            let path = item.path();
            if path.is_dir() {
                queue.push(path);
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

/// 把 `input` 的每个条目解到已存在的 `workspace`
pub fn extract_archive(input: &Path, workspace: &Path, limits: &Limits) -> Result<Extraction, String> {
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

    let mut signed = false;
    let mut extracted: usize = 0;
    let mut expanded_bytes: u64 = 0;

    for position in 0..archive.len() {
        let mut entry = archive
            .by_index(position)
            .map_err(|e| format!("unable to open archive entry #{position}: {e}"))?;
        let raw_name = entry.name().to_string();
        let segments = entry_segments(&raw_name)?;
        if segments.is_empty() {
            continue;
        }

        extracted += 1;
        if extracted > limits.entries {
            return Err("the archive holds more entries than the safety limit".to_string());
        }

        let size = entry.size();
        expanded_bytes = expanded_bytes.saturating_add(size);
        if size > limits.entry_bytes {
            return Err(format!("archive entry is too large: {raw_name}"));
        }
        if expanded_bytes > limits.total_bytes {
            return Err("the archive expands beyond the safety limit".to_string());
        }
        let compressed = entry.compressed_size();
        if compressed > 0 && size as f64 / compressed as f64 > limits.compression_ratio {
            return Err(format!(
                "archive entry has a suspicious compression ratio: {raw_name}"
            ));
        }
        if entry.is_symlink() {
            return Err(format!("archive entry is a symbolic link: {raw_name}"));
        }
        if is_signature_entry(&raw_name) {
            signed = true;
        }
        if entry.is_dir() || raw_name.ends_with('/') {
            continue;
        }

        let target = resolve_inside(workspace, &raw_name)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("unable to create a workspace directory for {raw_name}: {e}"))?;
        }
        let mut sink = File::create(&target)
            .map_err(|e| format!("unable to create the workspace file for {raw_name}: {e}"))?;
        std::io::copy(&mut entry, &mut sink)
            .map_err(|e| format!("unable to extract {raw_name}: {e}"))?;
    }

    WorkspaceMeta {
        format: 1,
        signed,
        entries: extracted,
    }
    .write(workspace)
    .map_err(|e| format!("unable to write the workspace metadata: {e}"))?;

    Ok(Extraction { signed, entries: extracted })
}

/// 把工作区重新打成 JAR；条目名取自相对路径
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

    // zip writer 需要持有临时文件句柄，因此走「原子写 + 闭包」
    crate::utils::fs::write_atomic_with(output, |temporary| {
        let handle =
            File::create(temporary).map_err(|e| format!("unable to create the output JAR: {e}"))?;
        let mut writer = zip::ZipWriter::new(handle);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

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
