//! 模组翻译：翻译记忆（跨模组缓存同源文本，JSON 落盘）

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::mod_translation::quality::validate_protected_tokens;
use crate::mod_translation::types::has_chinese;

/// 条目数上限，超出按 (updated_at 降序, hits 降序) 截断
const MAX_ENTRIES: usize = 100_000;

/// 单条翻译记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub translation: String,
    pub updated_at: u64,
    pub hits: u64,
}

/// 落盘格式（带版本号便于后续迁移）
#[derive(Debug, Serialize, Deserialize)]
struct MemoryFile {
    version: u32,
    entries: HashMap<String, MemoryEntry>,
}

/// 翻译记忆：key = SHA256(mod_ids + namespace + source)
#[derive(Debug)]
pub struct TranslationMemory {
    pub path: PathBuf,
    pub entries: HashMap<String, MemoryEntry>,
    pub dirty: bool,
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

    /// 记忆键：mod_ids 排序去重 + namespace 小写 + source
    pub fn memory_key(mod_ids: &[String], namespace: &str, source: &str) -> String {
        let mut ids = mod_ids.to_vec();
        ids.sort();
        ids.dedup();
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&ids).unwrap_or_default());
        hasher.update(namespace.to_ascii_lowercase().as_bytes());
        hasher.update(source.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 命中须过中文 + 占位符校验，不过视为 miss
    pub fn lookup(&mut self, mod_ids: &[String], namespace: &str, source: &str) -> Option<String> {
        let key = Self::memory_key(mod_ids, namespace, source);
        let entry = self.entries.get_mut(&key)?;
        let translation = entry.translation.trim().to_string();
        if translation.is_empty()
            || !has_chinese(&translation)
            || validate_protected_tokens(source, &translation).is_some()
        {
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

    /// 落盘：dirty 才写；超限截断；临时文件 + rename 原子写
    pub fn flush(&mut self) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }
        if self.entries.len() > MAX_ENTRIES {
            let mut entries = self.entries.drain().collect::<Vec<_>>();
            entries.sort_by(|left, right| {
                right
                    .1
                    .updated_at
                    .cmp(&left.1.updated_at)
                    .then_with(|| right.1.hits.cmp(&left.1.hits))
            });
            entries.truncate(MAX_ENTRIES);
            self.entries = entries.into_iter().collect();
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("无法创建翻译记忆目录: {e}"))?;
        }
        let file = MemoryFile {
            version: 1,
            entries: self.entries.clone(),
        };
        let content =
            serde_json::to_string(&file).map_err(|e| format!("翻译记忆序列化失败: {e}"))?;
        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, content).map_err(|e| format!("写入翻译记忆失败: {e}"))?;
        std::fs::rename(&temporary, &self.path).map_err(|e| format!("移动翻译记忆失败: {e}"))?;
        self.dirty = false;
        Ok(())
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

/// 记忆文件路径：`workspace_root/translation-memory-v1.json`
pub fn memory_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join("translation-memory-v1.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_key_normalizes_mod_ids_and_namespace() {
        let key_a = TranslationMemory::memory_key(
            &[
                "mod-b".to_string(),
                "mod-a".to_string(),
                "mod-b".to_string(),
            ],
            "Namespace",
            "Hello %s",
        );
        let key_b = TranslationMemory::memory_key(
            &["mod-a".to_string(), "mod-b".to_string()],
            "namespace",
            "Hello %s",
        );
        assert_eq!(key_a, key_b);
        assert_eq!(key_a.len(), 64);
    }

    #[test]
    fn lookup_rejects_invalid_translation() {
        let mut memory = TranslationMemory::load(PathBuf::from("unused.json"));
        let mod_ids = vec!["mod-a".to_string()];
        memory.record(&mod_ids, "ns", "Hello %s", "你好 %d".to_string());
        assert!(memory.lookup(&mod_ids, "ns", "Hello %s").is_none());
        memory.record(&mod_ids, "ns", "Hello %s", "你好 %s".to_string());
        assert_eq!(
            memory.lookup(&mod_ids, "ns", "Hello %s"),
            Some("你好 %s".to_string())
        );
    }

    #[test]
    fn flush_persists_and_reloads() {
        let path = std::env::temp_dir().join(format!(
            "mo-translation-memory-test-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let mut memory = TranslationMemory::load(path.clone());
        memory.record(&["mod-a".to_string()], "ns", "Hello", "你好".to_string());
        memory.flush().unwrap();
        let mut reloaded = TranslationMemory::load(path.clone());
        assert_eq!(
            reloaded.lookup(&["mod-a".to_string()], "ns", "Hello"),
            Some("你好".to_string())
        );
        let _ = std::fs::remove_file(&path);
    }
}
