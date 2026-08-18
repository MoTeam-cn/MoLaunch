//! 语言翻译路由单元测试（经 translate_lang.rs 的 #[path] 子模块引入）

use super::*;
use crate::mod_translation::types::{Loader, Quote};

fn sample_inspection() -> JarInspection {
    JarInspection {
        input_path: std::path::PathBuf::from("demo.jar"),
        original_filename: "demo.jar".to_string(),
        loader: Loader::Fabric,
        mod_ids: vec!["demo".to_string()],
        project_names: vec!["Demo".to_string()],
        version: None,
        signed: false,
        language_sources: Vec::new(),
        language_entries: 0,
        class_candidates: Vec::new(),
        coverage: Vec::new(),
        quote: Quote {
            estimated_input_tokens: 0,
            estimated_output_tokens: 0,
            estimated_tokens: 0,
            estimated_calls: 0,
            language_batches: 0,
            class_batches: 0,
            points: 0,
            characters: 0,
            entries: 0,
        },
        mod_name: None,
        existing_chinese: Vec::new(),
        warnings: Vec::new(),
    }
}

fn sample_source() -> LanguageSource {
    LanguageSource {
        kind: LanguageKind::Json,
        namespace: "demo".to_string(),
        source_path: "assets/demo/lang/en_us.json".to_string(),
        target_path: "assets/demo/lang/zh_cn.json".to_string(),
        entries: BTreeMap::from([
            ("demo.hello".to_string(), "Hello".to_string()),
            ("demo.blank".to_string(), "   ".to_string()),
            (
                "demo.url".to_string(),
                "https://example.com/foo".to_string(),
            ),
            ("demo.done".to_string(), "Sword".to_string()),
        ]),
        existing_target: BTreeMap::from([("demo.done".to_string(), "剑".to_string())]),
    }
}

#[test]
fn requires_work_filters_passthrough_and_translated_entries() {
    let source = sample_source();
    let entries: Vec<(String, String)> = source
        .entries
        .iter()
        .filter(|(key, src)| {
            quality::requires_work(
                key,
                src,
                source.existing_target.get(*key).map(String::as_str),
            )
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, vec!["demo.hello"]);
}

#[test]
fn evaluate_rejects_placeholder_mismatch_and_empty() {
    assert!(evaluate_translation("Spawn %d zombies", "生成 %s 只僵尸").is_err());
    assert_eq!(
        evaluate_translation("Spawn %d zombies", "生成 %d 只僵尸").unwrap(),
        "生成 %d 只僵尸"
    );
    assert!(evaluate_translation("Hello", "").is_err());
    assert!(evaluate_translation("Hello", "你好").is_ok());
}

#[test]
fn memory_hit_short_circuits_ai_call() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime.block_on(async {
        let mut memory = TranslationMemory::load(std::path::PathBuf::from("unused.json"));
        let mod_ids = vec!["demo".to_string()];
        let source = sample_source();
        memory.record(&mod_ids, "demo", "Hello", "你好".to_string());
        let batch = vec![("demo.hello".to_string(), "Hello".to_string())];
        let mut graph = WorkGraph::new("task-memory".to_string());
        let cancel = AtomicBool::new(false);
        let config = ai_core::AiConfig::default();
        // 全部命中记忆：不触发 AI 调用，直接返回命中译文
        let accepted = translate_batch(
            &sample_inspection(),
            &source,
            &batch,
            &config,
            "model",
            &mut memory,
            &mut graph,
            &cancel,
            &|_, _, _| {},
            0.0,
            100.0,
        )
        .await
        .unwrap();
        assert_eq!(accepted.get("demo.hello").map(String::as_str), Some("你好"));
        assert_eq!(graph.progress(), (1.0, 1.0));
    });
}
