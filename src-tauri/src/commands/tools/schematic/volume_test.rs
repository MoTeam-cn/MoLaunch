//! 体素打包测试：压缩往返、空气索引、超限拒绝。

use std::io::Read;

use crate::commands::tools::schematic::{read::SchematicData, session, volume};

fn sample() -> SchematicData {
    SchematicData {
        format: "sponge_v2",
        name: "demo".to_string(),
        size: [3, 2, 4],
        palette: vec![
            "minecraft:air".to_string(),
            "minecraft:stone".to_string(),
            "minecraft:jungle_slab[half=top]".to_string(),
        ],
        blocks: vec![
            0, 1, 1, 2, 0, 1, 0, 0, 2, 2, 1, 0, 0, 2, 2, 1, 1, 0, 2, 0, 0, 1, 2, 2,
        ],
        block_entities: Vec::new(),
        raw_entities: Vec::new(),
    }
}

#[test]
fn test_volume_roundtrip_keeps_block_indices() {
    let id = session::create(sample());
    let packed = volume::build(&id).expect("打包失败");
    assert_eq!(packed.size, [3, 2, 4]);
    assert_eq!(packed.total, 24);
    assert_eq!(packed.air_index, 0);
    assert_eq!(packed.non_air, 15);

    let raw = base64_decode(&packed.blocks_base64);
    let mut decoder = flate2::read::ZlibDecoder::new(raw.as_slice());
    let mut plain = Vec::new();
    decoder.read_to_end(&mut plain).expect("解压失败");
    assert_eq!(plain.len(), 48, "每个方块 2 字节");

    let restored: Vec<u16> = plain
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    assert_eq!(restored, sample().blocks);
    session::close(&id);
}

#[test]
fn test_volume_keeps_blockstate_properties_in_palette() {
    let id = session::create(sample());
    let packed = volume::build(&id).expect("打包失败");
    assert!(packed
        .palette
        .iter()
        .any(|entry| entry.as_str() == "minecraft:jungle_slab[half=top]"));
    session::close(&id);
}

#[test]
fn test_volume_rejects_unknown_session() {
    assert!(volume::build("sch-missing").is_err());
}

fn base64_decode(text: &str) -> Vec<u8> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(text)
        .expect("base64 解码失败")
}
