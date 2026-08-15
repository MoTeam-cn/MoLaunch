//! NBT 解析/保存/mca 重打包的单元测试

use std::io::Write;

use fastnbt::{SerOpts, Value as NbtValue};

use crate::commands::tools::nbt::convert::node_to_value;
use crate::commands::tools::nbt::mca::{parse_mca, save_mca_chunk};
use crate::commands::tools::types::NbtNode;

/// 构造叶子节点（测试辅助）
fn leaf(name: &str, tag_type: &str, value: serde_json::Value) -> NbtNode {
    NbtNode {
        name: name.to_string(),
        tag_type: tag_type.to_string(),
        value: Some(value),
        children: Vec::new(),
    }
}

/// 节点 → Value → 字节 → Value → 节点的完整往返
#[test]
fn node_round_trip() {
    let root = NbtNode {
        name: String::new(),
        tag_type: "compound".to_string(),
        value: None,
        children: vec![
            leaf("LevelName", "string", serde_json::json!("world")),
            leaf("Time", "long", serde_json::json!(12345)),
            leaf("RainTime", "int", serde_json::json!(0)),
            leaf("Color", "byte_array", serde_json::json!([1, 2, 3])),
            leaf("Hardcore", "byte", serde_json::json!(1)),
            NbtNode {
                name: "Dimensions".to_string(),
                tag_type: "compound".to_string(),
                value: None,
                children: vec![NbtNode {
                    name: "minecraft:overworld".to_string(),
                    tag_type: "compound".to_string(),
                    value: None,
                    children: vec![],
                }],
            },
        ],
    };

    let value = node_to_value(&root).unwrap();
    let bytes = fastnbt::to_bytes_with_opts(&value, SerOpts::new().root_name(&root.name)).unwrap();
    let back: NbtValue = fastnbt::from_bytes(&bytes).unwrap();
    match back {
        NbtValue::Compound(m) => {
            assert_eq!(m.len(), 6);
            assert!(matches!(m.get("LevelName"), Some(NbtValue::String(s)) if s == "world"));
            assert!(matches!(m.get("Time"), Some(NbtValue::Long(12345))));
            assert!(matches!(m.get("RainTime"), Some(NbtValue::Int(0))));
            assert!(matches!(m.get("Color"), Some(NbtValue::ByteArray(a)) if a.len() == 3));
            assert!(matches!(m.get("Hardcore"), Some(NbtValue::Byte(1))));
        }
        _ => panic!("根应为 compound"),
    }
}

/// 空 compound 也可正常序列化（TAG_End 结尾）
#[test]
fn empty_compound_round_trip() {
    let root = NbtNode {
        name: String::new(),
        tag_type: "compound".to_string(),
        value: None,
        children: vec![],
    };
    let value = node_to_value(&root).unwrap();
    let bytes = fastnbt::to_bytes_with_opts(&value, SerOpts::new().root_name(&root.name)).unwrap();
    let back: NbtValue = fastnbt::from_bytes(&bytes).unwrap();
    assert!(matches!(back, NbtValue::Compound(m) if m.is_empty()));
}

/// 构造 mca：一个 zlib 压缩的区块写入第 10 个位置
fn build_mca(chunk_root: &NbtNode) -> Vec<u8> {
    let value = node_to_value(chunk_root).unwrap();
    let nbt = fastnbt::to_bytes_with_opts(&value, SerOpts::new().root_name("")).unwrap();
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&nbt).unwrap();
    let compressed = encoder.finish().unwrap();

    let index = 10usize;
    let sector_offset = 2usize;
    let mut chunk_data = Vec::with_capacity(compressed.len() + 5);
    chunk_data.extend_from_slice(&((compressed.len() + 1) as u32).to_be_bytes());
    chunk_data.push(2); // zlib
    chunk_data.extend_from_slice(&compressed);
    let sector_count = chunk_data.len().div_ceil(512);

    let mut out = vec![0u8; 8192 + sector_count * 512];
    out[index * 4 + 2] = sector_offset as u8; // 扇区偏移 2（3 字节大端）
    out[index * 4 + 3] = sector_count as u8;
    out[sector_offset * 512..sector_offset * 512 + chunk_data.len()].copy_from_slice(&chunk_data);
    out
}

/// mca 解析 → 保存（重打包）→ 再解析，验证区块可编辑
#[test]
fn mca_parse_and_save() {
    let chunk_root = NbtNode {
        name: String::new(),
        tag_type: "compound".to_string(),
        value: None,
        children: vec![
            leaf("Level", "int", serde_json::json!(5)),
            leaf("xPos", "int", serde_json::json!(100)),
        ],
    };
    let raw = build_mca(&chunk_root);
    let chunks = parse_mca(&raw).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].index, 10);
    assert_eq!(chunks[0].root.children.len(), 2);

    let mut edited = NbtNode {
        name: String::new(),
        tag_type: "compound".to_string(),
        value: None,
        children: vec![
            leaf("Level", "int", serde_json::json!(99)),
            leaf("xPos", "int", serde_json::json!(100)),
        ],
    };
    // 额外加一个字段，验证重打包后新增字段可读
    edited
        .children
        .push(leaf("Sections", "list", serde_json::Value::Null));
    edited.children.last_mut().unwrap().children = vec![leaf("", "int", serde_json::json!(0))];

    let tmp = std::env::temp_dir().join("mol_test_chunk.mca");
    std::fs::write(&tmp, &raw).unwrap();
    save_mca_chunk(tmp.to_str().unwrap(), 10, &edited).unwrap();
    let after = std::fs::read(&tmp).unwrap();
    let chunks2 = parse_mca(&after).unwrap();
    assert_eq!(chunks2.len(), 1);
    let level = chunks2[0]
        .root
        .children
        .iter()
        .find(|c| c.name == "Level")
        .unwrap();
    assert_eq!(level.value.as_ref().unwrap().as_i64(), Some(99));
    std::fs::remove_file(&tmp).ok();
}

/// 非法 NBT 类型转换应报错
#[test]
fn invalid_node_type() {
    let bad = leaf("x", "unknown_type", serde_json::json!(1));
    assert!(node_to_value(&bad).is_err());
}
