//! 投影工具测试：编解码、统计、三种格式的写出与回读。

use std::path::PathBuf;

use super::codec::{
    count_non_air, decode_packed_longs, decode_varints, encode_blocks_base64, encode_packed_longs,
    encode_varints, is_air, normalize_block_id, top_counts,
};
use super::legacy_blocks::{legacy_block_count, legacy_block_name};
use super::read::{read_schematic, SchematicData};
use super::session;
use super::write::export;

fn sample_data() -> SchematicData {
    SchematicData {
        format: "sponge_v2",
        name: "demo".to_string(),
        size: [3, 2, 4],
        palette: vec![
            "minecraft:air".to_string(),
            "minecraft:stone".to_string(),
            "minecraft:oak_planks".to_string(),
        ],
        // 长度 = 3 * 2 * 4 = 24，x 最快
        blocks: vec![
            0, 1, 1, 2, 0, 1, 0, 0, 2, 2, 1, 0, 0, 2, 2, 1, 1, 0, 2, 0, 0, 1, 2, 2,
        ],
    }
}

fn temp_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("molaunch-schematic-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir.join(name)
}

#[test]
fn test_varints_roundtrip() {
    let values = vec![0u32, 1, 127, 128, 300, 65535];
    let encoded = encode_varints(&values);
    let decoded = decode_varints(&encoded).unwrap_or_default();
    assert_eq!(decoded, values);
}

#[test]
fn test_varints_rejects_truncated_sequence() {
    // 0x80 表示后续还有字节，但数据到此结束
    assert!(decode_varints(&[0x80]).is_err());
}

#[test]
fn test_packed_longs_roundtrip() {
    let values: Vec<u32> = vec![0, 1, 2, 3, 1, 0, 2, 1, 3, 2, 0, 1];
    let packed = encode_packed_longs(&values, 2);
    let decoded = decode_packed_longs(&packed, 2, values.len());
    assert_eq!(decoded, values);
}

#[test]
fn test_packed_longs_roundtrip_with_five_bits() {
    let values: Vec<u32> = (0..40).map(|v| v % 20).collect();
    let packed = encode_packed_longs(&values, 5);
    let decoded = decode_packed_longs(&packed, 5, values.len());
    assert_eq!(decoded, values);
}

#[test]
fn test_normalize_block_id_strips_properties() {
    assert_eq!(
        normalize_block_id("oak_stairs[facing=east,half=top]"),
        "minecraft:oak_stairs"
    );
    assert_eq!(normalize_block_id("minecraft:stone"), "minecraft:stone");
    assert_eq!(normalize_block_id("  "), "minecraft:air");
}

#[test]
fn test_is_air_matches_all_air_variants() {
    assert!(is_air("minecraft:air"));
    assert!(is_air("minecraft:cave_air"));
    assert!(is_air("minecraft:void_air"));
    assert!(!is_air("minecraft:stone"));
}

#[test]
fn test_top_counts_sorts_and_skips_air() {
    let palette = vec![
        "minecraft:air".to_string(),
        "minecraft:stone".to_string(),
        "minecraft:oak_planks".to_string(),
    ];
    let blocks = vec![0u16, 1, 1, 1, 2, 2];
    let counts = top_counts(&blocks, &palette, 10);
    assert_eq!(counts.len(), 2);
    assert_eq!(counts[0].0, "minecraft:stone");
    assert_eq!(counts[0].1, 3);
    assert_eq!(counts[1].0, "minecraft:oak_planks");
}

#[test]
fn test_count_non_air_ignores_air_entries() {
    let palette = vec!["minecraft:air".to_string(), "minecraft:stone".to_string()];
    let blocks = vec![0u16, 1, 1, 0];
    assert_eq!(count_non_air(&blocks, &palette), 2);
}

#[test]
fn test_encode_blocks_base64_matches_byte_length() {
    let blocks = vec![1u16, 2, 3];
    let encoded = encode_blocks_base64(&blocks);
    // 3 个 u16 = 6 字节，base64 后 8 个字符（无填充）
    assert_eq!(encoded.len(), 8);
}

#[test]
fn test_export_sponge_roundtrip() {
    let data = sample_data();
    let path = temp_path("roundtrip.schem");
    let written = export(&data, "sponge_v2", &path).unwrap_or(0);
    assert!(written > 0);

    let reloaded = read_schematic(&path).expect("回读 Sponge 投影失败");
    assert_eq!(reloaded.size, data.size);
    assert_eq!(reloaded.blocks, data.blocks);
    assert_eq!(reloaded.palette, data.palette);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_export_litematic_roundtrip() {
    let data = sample_data();
    let path = temp_path("roundtrip.litematic");
    let written = export(&data, "litematic", &path).unwrap_or(0);
    assert!(written > 0);

    let reloaded = read_schematic(&path).expect("回读 Litematica 投影失败");
    assert_eq!(reloaded.size, data.size);
    // 调色板顺序可能不同，按 id 还原后比较
    let mapped: Vec<String> = reloaded
        .blocks
        .iter()
        .map(|&index| {
            reloaded
                .palette
                .get(index as usize)
                .cloned()
                .unwrap_or_default()
        })
        .collect();
    let expected: Vec<String> = data
        .blocks
        .iter()
        .map(|&index| {
            data.palette
                .get(index as usize)
                .cloned()
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(mapped, expected);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_export_structure_roundtrip() {
    let data = sample_data();
    let path = temp_path("roundtrip.nbt");
    let written = export(&data, "structure", &path).unwrap_or(0);
    assert!(written > 0);

    let reloaded = read_schematic(&path).expect("回读结构文件失败");
    assert_eq!(reloaded.size, data.size);
    let mapped: Vec<String> = reloaded
        .blocks
        .iter()
        .map(|&index| {
            reloaded
                .palette
                .get(index as usize)
                .cloned()
                .unwrap_or_default()
        })
        .collect();
    let expected: Vec<String> = data
        .blocks
        .iter()
        .map(|&index| {
            data.palette
                .get(index as usize)
                .cloned()
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(mapped, expected);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_export_rejects_unknown_format() {
    let data = sample_data();
    let path = temp_path("unknown.bin");
    assert!(export(&data, "nope", &path).is_err());
}

#[test]
fn test_legacy_block_table_covers_common_blocks() {
    assert!(legacy_block_count() > 400);
    assert_eq!(legacy_block_name(1, 0), Some("minecraft:stone"));
    assert_eq!(legacy_block_name(1, 1), Some("minecraft:granite"));
    assert_eq!(legacy_block_name(5, 2), Some("minecraft:birch_planks"));
    assert_eq!(legacy_block_name(35, 14), Some("minecraft:red_wool"));
    assert_eq!(
        legacy_block_name(95, 3),
        Some("minecraft:light_blue_stained_glass")
    );
    assert_eq!(legacy_block_name(251, 15), Some("minecraft:black_concrete"));
    // 未收录的组合必须返回 None，由调用方退化为 legacy_<id>_<meta>
    assert_eq!(legacy_block_name(9999, 0), None);
}

#[test]
fn test_session_edit_undo_redo() {
    session::clear_all();
    let id = session::create(sample_data());

    // 2×2 区域里有 1 格本来就是目标方块，因此实际改动 3 格
    let (changed, non_air, can_undo, _) = session::edit(
        &id,
        session::EditRequest {
            axis: "y",
            layer: 0,
            x: 0,
            y: 0,
            w: 2,
            h: 2,
            block: 2,
            mode: "fill",
            from: None,
        },
    )
    .expect("编辑失败");
    assert_eq!(changed, 3);
    assert!(non_air > 0);
    assert!(can_undo);

    let (_, _, can_undo_after, can_redo_after) = session::history(&id, "undo").expect("撤销失败");
    assert!(!can_undo_after);
    assert!(can_redo_after);

    let (_, _, _, _) = session::history(&id, "redo").expect("重做失败");
    session::close(&id);
}

#[test]
fn test_session_slice_matches_layout() {
    session::clear_all();
    let id = session::create(sample_data());
    let (width, height, cells) = session::slice(&id, "y", 0, 0, 0, 3, 4).expect("切片失败");
    assert_eq!(width, 3);
    assert_eq!(height, 4);
    // 3×4 个格子，base64 长度应为 24 字节的 4/3 倍
    assert_eq!(cells.len(), 32);
    session::close(&id);
}

#[test]
fn test_session_transform_rotates_size() {
    session::clear_all();
    let id = session::create(sample_data());
    let (size, _) = session::transform(&id, "rotate_cw").expect("旋转失败");
    assert_eq!(size, [4, 2, 3]);
    let (restored, _) = session::transform(&id, "rotate_ccw").expect("反向旋转失败");
    assert_eq!(restored, [3, 2, 4]);
    session::close(&id);
}

#[test]
fn test_session_transform_rejects_unknown_kind() {
    session::clear_all();
    let id = session::create(sample_data());
    assert!(session::transform(&id, "spin").is_err());
    session::close(&id);
}

#[test]
fn test_session_slice_rejects_out_of_range_layer() {
    session::clear_all();
    let id = session::create(sample_data());
    assert!(session::slice(&id, "y", 99, 0, 0, 1, 1).is_err());
    session::close(&id);
}
