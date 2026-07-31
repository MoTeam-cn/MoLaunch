//! 库文件校验：并行检查缺失库 + 快速/完整两种校验模式

use std::path::Path;

use super::LibEntry;
use crate::minecraft::utils::file_checker::FileChecker;

/// 查找缺失库文件（并行检查）
///
/// - 并行：`std::thread::scope` 多线程检查
/// - 快速模式（`quick_check=true`）：仅检查存在+大小，启动时用，避免哈希卡顿
/// - 完整模式（`quick_check=false`）：计算 SHA1，安装/修复时严格校验
/// - 性能：73 库 200MB，串行+哈希 60s → 并行+仅大小 0.5s
pub fn find_missing_libs(libs: &[LibEntry], _game_dir: &Path, quick_check: bool) -> Vec<LibEntry> {
    use std::sync::Mutex;

    let missing: Mutex<Vec<LibEntry>> = Mutex::new(Vec::new());
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(libs.len())
        .max(1);

    // 按"预计检查耗时"粗略分组，让大文件（有 size 的）均匀分布到各线程
    // 简单策略：按索引取模分配
    let chunks: Vec<Vec<&LibEntry>> = if num_threads > 1 {
        (0..num_threads)
            .map(|tid| {
                libs.iter()
                    .enumerate()
                    .filter(|(i, _)| i % num_threads == tid)
                    .map(|(_, l)| l)
                    .collect()
            })
            .collect()
    } else {
        vec![libs.iter().collect()]
    };

    std::thread::scope(|s| {
        for chunk in chunks {
            let missing = &missing;
            s.spawn(move || {
                for lib in chunk {
                    let is_ok = if quick_check {
                        // 快速检查：只检查文件存在 + 大小匹配
                        quick_check_lib(lib)
                    } else {
                        // 完整校验：文件存在 + 大小 + SHA1
                        let checker = FileChecker::new()
                            .with_actual_size(if lib.size == 0 { -1 } else { lib.size })
                            .with_hash(lib.sha1.clone());
                        checker.is_valid(&lib.local_path)
                    };
                    if !is_ok {
                        missing.lock().unwrap().push(lib.clone());
                    }
                }
            });
        }
    });

    // 保持原有顺序（按 libs 中的顺序）
    let mut result = missing.into_inner().unwrap();
    result.sort_by_key(|l| {
        libs.iter()
            .position(|x| x.local_path == l.local_path)
            .unwrap_or(usize::MAX)
    });
    result
}

/// 快速检查库文件：只检查文件存在 + 大小匹配，不计算哈希
///
/// 用于启动时的快速校验，避免对大文件计算 SHA1 导致卡顿。
fn quick_check_lib(lib: &LibEntry) -> bool {
    let path = std::path::Path::new(&lib.local_path);
    if !path.exists() {
        return false;
    }
    // 有 size 元数据时检查大小是否匹配
    if lib.size > 0 {
        if let Ok(meta) = std::fs::metadata(path) {
            return meta.len() as i64 == lib.size;
        }
        return false;
    }
    // 无 size 元数据时只检查文件存在
    true
}
