//! 模组翻译：翻译记忆（跨模组复用同源文本）
//!
//! 落盘为「行数组」：每行 [键, 译文, 更新时间, 命中数]，避免对象键名开销；
//! 键 = sha256(长度前缀拼接的 modId 列表 + 小写命名空间 + 原文)。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::mod_translation::quality::validate_protected_tokens;
use crate::mod_translation::types::has_chinese;

/// 落盘文件名
const STORE_NAME: &str = "translation-memory-v1.json";
/// 落盘格式版本
const STORE_VERSION: u32 = 1;
/// 行数上限，超出按（更新时间降序, 命中数降序）截断
const ROW_CAP: usize = 100_000;

/// 单行记忆（序列化为四元数组）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Row(String, String, u64, u64);

impl Row {
    fn text(&self) -> &str {
        &self.1
    }

    fn stamp(&self) -> u64 {
        self.2
    }

    fn uses(&self) -> u64 {
        self.3
    }

    fn touch(&mut self) {
        self.2 = now_seconds();
        self.3 += 1;
    }
}

/// 落盘结构
#[derive(Debug, Serialize, Deserialize)]
struct Store {
    v: u32,
    rows: Vec<Row>,
}

/// 翻译记忆
#[derive(Debug)]
pub struct TranslationMemory {
    pub path: PathBuf,
    pub dirty: bool,
    rows: BTreeMap<String, Row>,
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

/// 记忆内容是否可用：非空、含中文、占位符一致
fn usable(source: &str, candidate: &str) -> bool {
    let trimmed = candidate.trim();
    !trimmed.is_empty()
        && has_chinese(trimmed)
        && validate_protected_tokens(source, trimmed).is_none()
}

impl TranslationMemory {
    /// 从磁盘加载；文件缺失或损坏时返回空记忆
    pub fn load(path: PathBuf) -> Self {
        let rows = std::fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Store>(&raw).ok())
            .map(|store| {
                store
                    .rows
                    .into_iter()
                    .map(|row| (row.0.clone(), row))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            path,
            dirty: false,
            rows,
        }
    }

    /// 记忆键：modId 排序去重 + 小写命名空间 + 原文，各段带长度前缀
    pub fn memory_key(mod_ids: &[String], namespace: &str, source: &str) -> String {
        let mut ids: Vec<&str> = mod_ids.iter().map(String::as_str).collect();
        ids.sort_unstable();
        ids.dedup();

        let mut hasher = Sha256::new();
        let mut feed = |part: &str| {
            hasher.update((part.len() as u32).to_le_bytes());
            hasher.update(part.as_bytes());
        };
        for id in ids {
            feed(id);
        }
        feed(&namespace.to_ascii_lowercase());
        feed(source);
        format!("{:x}", hasher.finalize())
    }

    /// 命中：校验通过才返回译文，并累加命中数
    pub fn lookup(&mut self, mod_ids: &[String], namespace: &str, source: &str) -> Option<String> {
        let key = Self::memory_key(mod_ids, namespace, source);
        let row = self.rows.get_mut(&key)?;
        if !usable(source, row.text()) {
            return None;
        }
        let text = row.text().trim().to_string();
        row.touch();
        self.dirty = true;
        Some(text)
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
        self.rows
            .insert(key.clone(), Row(key, translation, now_seconds(), 0));
        self.dirty = true;
    }

    /// 超出上限时按热度截断
    fn trim(&mut self) {
        if self.rows.len() <= ROW_CAP {
            return;
        }
        let mut ranked: Vec<(u64, u64, String)> = self
            .rows
            .iter()
            .map(|(key, row)| (row.stamp(), row.uses(), key.clone()))
            .collect();
        ranked.sort_unstable_by(|left, right| right.0.cmp(&left.0).then(right.1.cmp(&left.1)));
        ranked.truncate(ROW_CAP);
        let keep: BTreeSet<String> = ranked.into_iter().map(|(_, _, key)| key).collect();
        self.rows.retain(|key, _| keep.contains(key));
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
        let store = Store {
            v: STORE_VERSION,
            rows: self.rows.values().cloned().collect(),
        };
        let payload =
            serde_json::to_string(&store).map_err(|e| format!("翻译记忆序列化失败: {e}"))?;
        crate::utils::fs::write_atomic(&self.path, payload.as_bytes())?;
        self.dirty = false;
        Ok(())
    }
}

/// 记忆文件路径
pub fn memory_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(STORE_NAME)
}

#[cfg(test)]
#[path = "memory_test.rs"]
mod tests;
