//! 安装产物完整性校验
//!
//! 注入器返回 0 不等于装好了：它内部的 processors 可能失败而不影响退出码。
//! 装完必须回头核对 version JSON 里声明的库是否真的在盘上——尤其是
//! `net.minecraft:client:<版本>:srg` 这类**只能由本地 processor 生成**、
//! 任何下载源都拿不到的文件。缺了它，游戏能启动但会立刻报安装损坏。

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::shared::maven_path_to_local;

/// 一个应当存在的库文件
pub(crate) struct LibraryFile {
    pub name: String,
    pub path: PathBuf,
    /// 只能由本地 processor 生成（没有下载地址）
    pub generated: bool,
}

/// 这个条目的文件只能本地生成吗
///
/// 判据是**有没有下载地址**，不是猜名字。早先用 `:srg` 之类的后缀判断，
/// 换个命名就漏判，缺文件从「致命」降级成「只记警告」。
pub(crate) fn is_locally_generated(library: &Value) -> bool {
    library
        .get("downloads")
        .and_then(|downloads| downloads.get("artifact"))
        .and_then(|artifact| artifact.get("url"))
        .and_then(|url| url.as_str())
        .is_none()
}

/// 从 version JSON 里取出所有应当存在的库文件
///
/// 有 `downloads.artifact.path` 就用它，没有就按 Maven 坐标推路径。
pub(crate) fn required_library_files(doc: &Value, game_dir: &Path) -> Vec<LibraryFile> {
    let mut files = Vec::new();
    let Some(libraries) = doc.get("libraries").and_then(|value| value.as_array()) else {
        return files;
    };
    for library in libraries {
        let Some(name) = library.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        // natives 走 classifier，单独一类，不在这里判
        if library.get("natives").is_some() {
            continue;
        }
        let declared = library
            .get("downloads")
            .and_then(|value| value.get("artifact"))
            .and_then(|value| value.get("path"))
            .and_then(|value| value.as_str());
        let path = match declared {
            Some(relative) => game_dir.join(relative),
            None => maven_path_to_local(name, game_dir),
        };
        files.push(LibraryFile {
            name: name.to_string(),
            path,
            generated: is_locally_generated(library),
        });
    }
    files
}

/// 校验一次安装的结果
///
/// 返回「缺失但不算致命」的库名，供调用方记录。
/// 本地生成的文件缺失视为致命：那种缺口只会在启动时报出来，离安装现场太远。
pub(crate) fn check_install(game_dir: &Path, json_path: &Path) -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string(json_path)
        .map_err(|e| format!("读不到版本 JSON（{}）：{}", json_path.display(), e))?;
    let doc: Value = serde_json::from_str(&text)
        .map_err(|e| format!("版本 JSON 解析失败（{}）：{}", json_path.display(), e))?;

    let mut fatal: Vec<String> = Vec::new();
    let mut soft: Vec<String> = Vec::new();
    for file in required_library_files(&doc, game_dir) {
        if file.path.is_file() {
            continue;
        }
        if file.generated {
            fatal.push(format!("{}（{}）", file.name, file.path.display()));
        } else {
            soft.push(file.name);
        }
    }

    if !soft.is_empty() {
        crate::log_warn!(
            "[Install] 安装后发现有 {} 个普通库文件缺失，可能是可选库：{:?}",
            soft.len(),
            soft
        );
    }

    if !fatal.is_empty() {
        return Err(format!(
            "安装不完整：缺少 {} 个需要本地生成的文件（{}）。\
             这类文件没有下载地址，只能由安装器在本地跑 processor 生成——\
             原因多见于 Java 版本与游戏版本不匹配、或临时目录没有写权限。\
             请确认已安装 Java 21 后重试。",
            fatal.len(),
            fatal.join("、")
        ));
    }
    Ok(soft)
}

#[cfg(test)]
#[path = "artifact_check_test.rs"]
mod artifact_check_test;
