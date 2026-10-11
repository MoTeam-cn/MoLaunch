//! 文件系统操作工具
//!
//! 消除 `std::fs::create_dir_all` / `std::fs::read_to_string` 重复的 map_err 样板，
//! 并提供日志/报告场景的通用读取（末尾 N 行、指定行段、最新文件）。

use std::path::Path;

/// 创建目录（含父目录）
pub fn ensure_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| format!("创建目录失败: {}", e))
}

/// 读取文件全部内容
pub fn read_to_string(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))
}

/// 取文本末尾 N 行（倒序取再反转，保证原顺序）
pub fn tail_lines(content: &str, lines: usize) -> String {
    content
        .lines()
        .rev()
        .take(lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

/// 读取文件末尾 N 行，超长时按字符数截断（保留行首，即最新内容）
pub fn read_tail(path: &Path, lines: usize, max_chars: usize) -> Result<String, String> {
    let content = read_to_string(path)?;
    let tail = tail_lines(&content, lines);
    Ok(crate::utils::format::truncate_chars(&tail, max_chars))
}

/// 目录中修改时间最新的文件（可按扩展名过滤，可选）
pub fn newest_file(dir: &Path, ext: Option<&str>) -> Option<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            ext.map(|e| p.extension().map(|x| x == e).unwrap_or(false))
                .unwrap_or(true)
        })
        .max_by_key(|p| p.metadata().and_then(|m| m.modified()).ok())
}

/// 递归复制目录内容到目标目录（目标不存在则创建，同名文件覆盖）
///
/// 统一替代 migrations/common.rs、version/pack_common/entries.rs、plugins/install.rs
/// 中逐字相同的三份实现。
///
/// 注意：本函数不做过滤、不返回计数。带 skip 集合与计数需求的增量合并
/// （如 frp/install/files.rs）语义不同，不要强行套用。
pub fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            std::fs::copy(&path, &dst_path).map_err(|e| format!("复制文件失败: {}", e))?;
        }
    }
    Ok(())
}

/// 递归统计目录总字节数（读不到的子项按 0 计，不返回错误）
///
/// 统一替代 version/pack_common/entries.rs、tools/archive/helpers.rs、
/// tools/resourcepack/list.rs、tools/resourcepack/explore/tree.rs 中
/// 逐字相同的四份实现。
pub fn dir_total_size(dir: &std::path::Path) -> u64 {
    let mut total: u64 = 0;
    let Ok(read) = std::fs::read_dir(dir) else {
        return 0;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            total += dir_total_size(&path);
        } else {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    total
}

/// 原子写入文件：先写同目录下的唯一临时文件，再 rename 覆盖目标。
///
/// 临时文件名带自增序号，避免并发写入共用同一 tmp 导致 rename 找不到源文件
/// （os error 2）。各调用点原先普遍用 with_extension("tmp") 的固定名，存在此问题。
///
/// 注意：仅保证同分区 rename 的原子性；不保证 fsync 落盘。
pub fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    write_atomic_with(path, |tmp| {
        std::fs::write(tmp, bytes).map_err(|e| format!("写入临时文件失败: {}", e))
    })
}

/// 原子写入文件（自定义写入过程）
///
/// 与 write_atomic 相同，但把「写临时文件」这一步交给 f——适用于需要在 rename 前
/// 对临时文件做额外处理（如补可执行位），或无法一次性拿到全部字节（如流式拼接、
/// 交由第三方 writer 落盘）的场景。f 收到的是临时文件路径。
///
/// 注意：f 失败时不会清理已生成的临时文件（与原各处手写实现的行为一致）。
pub fn write_atomic_with<F>(path: &std::path::Path, f: F) -> Result<(), String>
where
    F: FnOnce(&std::path::Path) -> Result<(), String>,
{
    let tmp = temp_path_for(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    f(&tmp)?;
    std::fs::rename(&tmp, path).map_err(|e| format!("替换目标文件失败: {}", e))
}

/// 生成目标文件对应的唯一临时路径（同目录、带自增序号）
fn temp_path_for(path: &std::path::Path) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static TMP_SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".tmp{}", seq));
    std::path::PathBuf::from(tmp)
}
