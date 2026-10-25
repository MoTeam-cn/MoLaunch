//! class 常量池 / 候选判定 / 改写 / 工作区往返测试（规格 §7.1 ~ §7.5）

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;

use super::classify::classify_class_text;
use super::discover::{deterministic_class_exclusion_reason, discover_class_candidates};
use super::pool::{class_string_constants, parse_class_constant_pool};
use super::rewrite::replace_class_utf8;
use crate::mod_translation::jar;

/// 极简 class 文件构造器：只拼常量池，够解析用
struct ClassBuilder {
    pool: Vec<u8>,
    slots: u16,
}

impl ClassBuilder {
    fn new() -> Self {
        Self {
            pool: Vec::new(),
            slots: 0,
        }
    }

    fn utf8(&mut self, text: &str) -> u16 {
        let bytes = text.as_bytes();
        self.pool.push(1);
        self.pool
            .extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        self.pool.extend_from_slice(bytes);
        self.slots += 1;
        self.slots
    }

    fn string(&mut self, utf8_slot: u16) -> u16 {
        self.pool.push(8);
        self.pool.extend_from_slice(&utf8_slot.to_be_bytes());
        self.slots += 1;
        self.slots
    }

    fn long(&mut self) -> u16 {
        self.pool.push(5);
        self.pool.extend_from_slice(&0u64.to_be_bytes());
        self.slots += 2;
        self.slots
    }

    fn double(&mut self) -> u16 {
        self.pool.push(6);
        self.pool.extend_from_slice(&0u64.to_be_bytes());
        self.slots += 2;
        self.slots
    }

    fn build(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xCAFE_BABEu32.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&52u16.to_be_bytes());
        bytes.extend_from_slice(&(self.slots + 1).to_be_bytes());
        bytes.extend_from_slice(&self.pool);
        bytes
    }
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let path = std::env::temp_dir().join(format!("molaunch-{tag}-{nanos}"));
    std::fs::create_dir_all(&path).expect("create temp dir");
    path
}

#[test]
fn test_pool_utf8_slots_and_spans_are_contiguous() {
    let mut builder = ClassBuilder::new();
    builder.utf8("hello");
    let slot = builder.utf8("world");
    builder.string(slot);
    let bytes = builder.build();

    let entries = parse_class_constant_pool(&bytes).expect("pool parses");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].0, "hello");
    assert_eq!(entries[1].0, "world");
    for (text, start, end) in &entries {
        assert_eq!(bytes[*start], 1, "span must start at the utf8 tag");
        assert_eq!(&bytes[start + 3..*end], text.as_bytes());
    }
}

#[test]
fn test_pool_strings_returns_only_referenced_text() {
    let mut builder = ClassBuilder::new();
    builder.utf8("unused");
    let used = builder.utf8("visible");
    builder.string(used);
    let bytes = builder.build();

    assert_eq!(class_string_constants(&bytes), vec!["visible".to_string()]);
}

#[test]
fn test_pool_two_slot_entries_keep_slot_numbers_aligned() {
    let mut builder = ClassBuilder::new();
    let first = builder.utf8("first");
    builder.long();
    builder.double();
    let last = builder.utf8("last");
    builder.string(last);
    let bytes = builder.build();

    let entries = parse_class_constant_pool(&bytes).expect("pool parses");
    let texts: Vec<&str> = entries.iter().map(|(text, _, _)| text.as_str()).collect();
    assert_eq!(texts, vec!["first", "last"]);
    assert_eq!(first, 1);
    assert_eq!(last, 6, "long/double each occupy two slots");
}

#[test]
fn test_pool_rejects_non_class_payload() {
    let error = parse_class_constant_pool(b"PK\x03\x04not-a-class").expect_err("must reject");
    assert!(error.contains("not a class file"));
}

#[test]
fn test_pool_rejects_truncated_bytes() {
    let mut builder = ClassBuilder::new();
    builder.utf8("truncated");
    let mut bytes = builder.build();
    bytes.truncate(bytes.len() - 3);
    assert!(parse_class_constant_pool(&bytes).is_err());
}

#[test]
fn test_pool_rejects_unknown_tag() {
    let mut bytes = ClassBuilder::new().build();
    // 在常量池末尾追加一个未知 tag 99
    let count_offset = 8;
    bytes[count_offset + 1] = 2;
    bytes.push(99);
    assert!(parse_class_constant_pool(&bytes).is_err());
}

#[test]
fn test_rewrite_replaces_every_matching_entry() {
    let mut builder = ClassBuilder::new();
    builder.utf8("Press any key");
    builder.utf8("Press any key");
    builder.utf8("keep me");
    let bytes = builder.build();

    let patched = replace_class_utf8(&bytes, "Press any key", "按任意键").expect("rewrite works");
    let texts: Vec<String> = parse_class_constant_pool(&patched)
        .expect("patched pool parses")
        .into_iter()
        .map(|(text, _, _)| text)
        .collect();
    assert_eq!(texts, vec!["按任意键", "按任意键", "keep me"]);
}

#[test]
fn test_rewrite_returns_input_when_text_is_absent() {
    let mut builder = ClassBuilder::new();
    builder.utf8("only entry");
    let bytes = builder.build();

    let patched = replace_class_utf8(&bytes, "missing", "替换").expect("rewrite works");
    assert_eq!(patched, bytes);
}

#[test]
fn test_rewrite_rejects_oversized_replacement() {
    let mut builder = ClassBuilder::new();
    builder.utf8("short");
    let bytes = builder.build();

    let oversized = "长".repeat(30_000);
    assert!(replace_class_utf8(&bytes, "short", &oversized).is_err());
}

#[test]
fn test_classify_detects_each_shape() {
    assert!(!classify_class_text("已经翻译").0);
    assert_eq!(classify_class_text("ab").1, "invalid");
    assert_eq!(classify_class_text("%s %s").1, "format_only");
    assert_eq!(classify_class_text("MAX_HEALTH").1, "constant");
    assert_eq!(classify_class_text("Ljava/lang/String;").1, "descriptor");
    assert_eq!(classify_class_text("(ID)V").1, "descriptor");
    assert_eq!(
        classify_class_text("assets/mymod/textures/item.png").1,
        "structural"
    );
    assert_eq!(classify_class_text("https://example.com/a").1, "structural");
    assert_eq!(classify_class_text("/give @p stone").1, "pure_command");
    assert_eq!(classify_class_text("Diamond").1, "display_word");
    assert_eq!(
        classify_class_text("Press any key to continue").1,
        "natural_language"
    );
}

#[test]
fn test_classify_treats_candidates_as_translatable() {
    assert!(classify_class_text("Press any key to continue").0);
    assert!(classify_class_text("Diamond").0);
    assert!(!classify_class_text("minecraft:stone").0);
}

#[test]
fn test_discover_aggregates_same_text_across_files() {
    let workspace = unique_dir("discover");
    let mut builder = ClassBuilder::new();
    let slot = builder.utf8("Press any key to continue");
    builder.string(slot);
    let bytes = builder.build();

    std::fs::create_dir_all(workspace.join("a")).expect("mkdir a");
    std::fs::create_dir_all(workspace.join("b")).expect("mkdir b");
    std::fs::write(workspace.join("a/One.class"), &bytes).expect("write a");
    std::fs::write(workspace.join("b/Two.class"), &bytes).expect("write b");

    let candidates = discover_class_candidates(&workspace);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].text, "Press any key to continue");
    assert_eq!(candidates[0].occurrences, 2);
    assert_eq!(candidates[0].paths.len(), 2);
    assert_eq!(candidates[0].id.len(), 24);

    let _ = std::fs::remove_dir_all(&workspace);
}

#[test]
fn test_deterministic_exclusions_cover_named_shapes() {
    assert_eq!(
        deterministic_class_exclusion_reason("a/B.class", "com.example.Foo"),
        Some("java_class_name")
    );
    assert_eq!(
        deterministic_class_exclusion_reason("a/B.class", "^\\d+$"),
        Some("regular_expression")
    );
    assert_eq!(
        deterministic_class_exclusion_reason("com/mod/file/Reader.class", "failed to read"),
        Some("internal_diagnostic")
    );
    assert_eq!(
        deterministic_class_exclusion_reason("com/mod/gui/Screen.class", "failed to read"),
        None
    );
    assert_eq!(
        deterministic_class_exclusion_reason("com/mod/file/Reader.class", "Reading data"),
        None
    );
}

fn build_archive(path: &Path, entries: &[(&str, &[u8])]) {
    let handle = File::create(path).expect("create archive");
    let mut writer = zip::ZipWriter::new(handle);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, content) in entries {
        writer.start_file(*name, options).expect("start entry");
        writer.write_all(content).expect("write entry");
    }
    writer.finish().expect("finish archive");
}

#[test]
fn test_extract_and_package_round_trip() {
    let root = unique_dir("roundtrip");
    let archive = root.join("input.jar");
    build_archive(
        &archive,
        &[
            ("assets/demo/lang/en_us.json", b"{}"),
            ("META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n"),
            ("META-INF/DEMO.SF", b"signature"),
        ],
    );

    let workspace = root.join("workspace");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    let extraction = jar::extract_archive(&archive, &workspace, &jar::Limits::default())
        .expect("extract succeeds");
    assert!(extraction.signed, "signature entry is detected");
    assert!(workspace.join("assets/demo/lang/en_us.json").is_file());
    assert!(
        !workspace.join("META-INF/DEMO.SF").exists(),
        "stale signatures are not extracted"
    );

    let meta = jar::WorkspaceMeta::read(&workspace).expect("meta is written");
    assert!(meta.signed);

    let output = root.join("output.jar");
    jar::package_archive(&workspace, &output).expect("package succeeds");
    assert!(output.is_file());
    assert!(
        jar::package_archive(&workspace, &output).is_err(),
        "existing output is not overwritten"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_extract_rejects_escaping_paths() {
    let root = unique_dir("escape");
    let archive = root.join("bad.jar");
    build_archive(&archive, &[("../evil.txt", b"x")]);

    let workspace = root.join("workspace");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    assert!(jar::extract_archive(&archive, &workspace, &jar::Limits::default()).is_err());

    let _ = std::fs::remove_dir_all(&root);
}
