//! 模组翻译：翻译记忆（跨模组复用同源文本，JSON 落盘）
//!
//! 记忆键 = sha256(排序去重后的 modId 列表 ␀ 小写命名空间 ␀ 原文)，
//! 命中时还要过一遍「含中文 + 占位符一致」才可用，否则视为未命中。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::mod_translation::quality::validate_protected_tokens;
use crate::mod_translation::types::has_chinese;

/// 条目数上限，超出按 (更新时间降序, 命中数降序) 截断
const MAX_ENTRIES: usize = 100_000;
/// 落盘文件名
const MEMORY_FILE: &str = "translation-memory-v1.json";
/// 落盘格式版本
const MEMORY_VERSION: u32 = 1;

/// 单条翻译记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub translation: String,
    pub updated_at: u64,
    pub hits: u64,
}

/// 落盘格式（带版本号便于迁移）
#[derive(Debug, Serialize, Deserialize)]
struct MemoryFile {
    version: u32,
    entries: HashMap<String, MemoryEntry>,
}

/// 翻译记忆
#[derive(Debug)]
pub struct TranslationMemory {
    pub path: PathBuf,
    pub entries: HashMap<String, MemoryEntry>,
    pub dirty: bool,
}

/// 记忆内容是否可用：非空、含中文、占位符一致
fn is_usable(source: &str, translation: &str) -> bool {
    let trimmed = translation.trim();
    !trimmed.is_empty()
        && has_chinese(trimmed)
        && validate_protected_tokens(source, trimmed).is_none()
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

impl TranslationMemory {
    /// 从磁盘加载；文件缺失或损坏时返回空记忆
    pub fn load(path: PathBuf) -> Self {
        let entries = std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<MemoryFile>(&content).ok())
            .map(|file| file.entries)
            .unwrap_or_default();
        Self {
            path,
            entries,
            dirty: false,
        }
    }

    /// 记忆键：modId 排序去重 + 小写命名空间 + 原文
    pub fn memory_key(mod_ids: &[String], namespace: &str, source: &str) -> String {
        let mut ids = mod_ids.to_vec();
        ids.sort();
        ids.dedup();
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&ids).unwrap_or_default());
        hasher.update(namespace.to_ascii_lowercase().as_bytes());
        hasher.update(source.as_bytes());
        let digest = format!("{:x}", hasher.finalize());
        digest
    }

    /// 命中：校验通过才返回译文，并累加命中数
    pub fn lookup(&mut self, mod_ids: &[String], namespace: &str, source: &str) -> Option<String> {
        let key = Self::memory_key(mod_ids, namespace, source);
        let entry = self.entries.get_mut(&key)?;
        let translation = entry.translation.trim().to_string();
        if !is_usable(source, &translation) {
            return None;
        }
        entry.hits += 1;
        entry.updated_at = now_seconds();
        self.dirty = true;
        Some(translation)
    }

    /// 记录一条翻译（覆盖同键旧值）
    pub fn record(
        &mut self,
        mod_ids: &[String],
        namespace: &str,
        source: &str,
        translation: String,
    ) {
        let key = Self::memory_key(mod_ids, namespace, source);
        self.entries.insert(
            key,
            MemoryEntry {
                translation,
                updated_at: now_seconds(),
                hits: 0,
            },
        );
        self.dirty = true;
    }

    /// 超出上限时按热度截断
    fn trim(&mut self) {
        if self.entries.len() <= MAX_ENTRIES {
            return;
        }
        let mut ranked: Vec<(String, MemoryEntry)> = self.entries.drain().collect();
        ranked.sort_by(|left, right| {
            right
                .1
                .updated_at
                .cmp(&left.1.updated_at)
                .then_with(|| right.1.hits.cmp(&left.1.hits))
        });
        ranked.truncate(MAX_ENTRIES);
        self.entries = ranked.into_iter().collect();
    }

    /// 落盘：dirty 才写，原子替换
    pub fn flush(&mut self) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }
        self.trim();
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("无法创建翻译记忆目录: {e}"))?;
        }
        let file = MemoryFile {
            version: MEMORY_VERSION,
            entries: self.entries.clone(),
        };
        let content =
            serde_json::to_string(&file).map_err(|e| format!("翻译记忆序列化失败: {e}"))?;
        crate::utils::fs::write_atomic(&self.path, content.as_bytes())?;
        self.dirty = false;
        Ok(())
    }
}

/// 记忆文件路径
pub fn memory_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(MEMORY_FILE)
}

#[cfg(test)]
#[path = "memory_test.rs"]
mod tests;
