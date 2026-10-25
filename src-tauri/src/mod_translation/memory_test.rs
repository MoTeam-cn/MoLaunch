//! 翻译记忆测试

use std::path::PathBuf;

use super::TranslationMemory;

#[test]
fn test_memory_key_normalizes_mod_ids_and_namespace() {
    let left = TranslationMemory::memory_key(
        &[
            "mod-b".to_string(),
            "mod-a".to_string(),
            "mod-b".to_string(),
        ],
        "Namespace",
        "Hello %s",
    );
    let right = TranslationMemory::memory_key(
        &["mod-a".to_string(), "mod-b".to_string()],
        "namespace",
        "Hello %s",
    );
    assert_eq!(left, right);
    assert_eq!(left.len(), 64);
}

#[test]
fn test_lookup_rejects_invalid_translation() {
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
fn test_flush_persists_and_reloads() {
    let path = std::env::temp_dir().join(format!(
        "mo-translation-memory-test-{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);

    let mut memory = TranslationMemory::load(path.clone());
    memory.record(&["mod-a".to_string()], "ns", "Hello", "你好".to_string());
    memory.flush().expect("flush");

    let mut reloaded = TranslationMemory::load(path.clone());
    assert_eq!(
        reloaded.lookup(&["mod-a".to_string()], "ns", "Hello"),
        Some("你好".to_string())
    );
    let _ = std::fs::remove_file(&path);
}
