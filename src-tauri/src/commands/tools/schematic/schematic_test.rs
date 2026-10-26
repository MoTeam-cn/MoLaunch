//! 投影工具测试：编解码、统计、三种格式的写出与回读。

use std::path::PathBuf;

use super::codec::{
    count_non_air, decode_packed_longs, decode_varints, encode_packed_longs, encode_varints,
    is_air, normalize_block_id, top_counts,
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
        block_entities: Vec::new(),
        raw_entities: Vec::new(),
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
    assert_eq!(legacy_block_name(1, 0).as_deref(), Some("minecraft:stone"));
    assert_eq!(
        legacy_block_name(1, 1).as_deref(),
        Some("minecraft:granite")
    );
    // 位规则：原木低 2 位树种、高 2 位轴向（3 表示全树皮）
    assert_eq!(
        legacy_block_name(17, 0).as_deref(),
        Some("minecraft:oak_log")
    );
    assert_eq!(
        legacy_block_name(17, 13).as_deref(),
        Some("minecraft:spruce_wood")
    );
    assert_eq!(
        legacy_block_name(162, 13).as_deref(),
        Some("minecraft:dark_oak_wood")
    );
    // 带釉陶瓦颜色由 id 决定，元数据是朝向
    assert_eq!(
        legacy_block_name(241, 0).as_deref(),
        Some("minecraft:pink_glazed_terracotta")
    );
    assert_eq!(
        legacy_block_name(244, 3).as_deref(),
        Some("minecraft:cyan_glazed_terracotta")
    );
    assert_eq!(
        legacy_block_name(5, 2).as_deref(),
        Some("minecraft:birch_planks")
    );
    assert_eq!(
        legacy_block_name(35, 14).as_deref(),
        Some("minecraft:red_wool")
    );
    assert_eq!(
        legacy_block_name(95, 3).as_deref(),
        Some("minecraft:light_blue_stained_glass")
    );
    assert_eq!(
        legacy_block_name(251, 15).as_deref(),
        Some("minecraft:black_concrete")
    );
    // 未收录的组合必须返回 None，由调用方退化为 legacy_<id>_<meta>
    assert_eq!(legacy_block_name(9999, 0), None);
    assert!(legacy_block_name(1, 0).is_some());
    assert_eq!(legacy_block_count() % 16, 0);
}

#[test]
fn test_session_edit_undo_redo() {
    let id = session::create(sample_data());

    // 2×2 区域里有 1 格本来就是目标方块，因此实际改动 3 格
    let (changed, non_air, can_undo, _) = session::edit(
        &id,
        session::EditRequest {
            shape: "rect",
            radius: 0,
            height: 0,
            points: &[],
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
    let id = session::create(sample_data());
    let (size, _) = session::transform(&id, "rotate_cw").expect("旋转失败");
    assert_eq!(size, [4, 2, 3]);
    let (restored, _) = session::transform(&id, "rotate_ccw").expect("反向旋转失败");
    assert_eq!(restored, [3, 2, 4]);
    session::close(&id);
}

#[test]
fn test_session_transform_rejects_unknown_kind() {
    let id = session::create(sample_data());
    assert!(session::transform(&id, "spin").is_err());
    session::close(&id);
}

#[test]
fn test_session_slice_rejects_out_of_range_layer() {
    let id = session::create(sample_data());
    assert!(session::slice(&id, "y", 99, 0, 0, 1, 1).is_err());
    session::close(&id);
}

#[test]
fn test_edit_sphere_fills_round_volume() {
    let id = session::create(sample_data());
    let (changed, _, _, _) = session::edit(
        &id,
        session::EditRequest {
            axis: "y",
            layer: 0,
            x: 1,
            y: 2,
            w: 1,
            h: 1,
            shape: "sphere",
            radius: 1,
            height: 0,
            points: &[],
            block: 2,
            mode: "fill",
            from: None,
        },
    )
    .expect("球体填充失败");
    // 半径 1 的球体共 7 格（含中心），其中与原方块相同的会被跳过
    assert!(changed > 0 && changed <= 7, "实际改动 {}", changed);
    session::close(&id);
}

#[test]
fn test_edit_cylinder_respects_height() {
    let id = session::create(sample_data());
    let (changed, _, _, _) = session::edit(
        &id,
        session::EditRequest {
            axis: "y",
            layer: 1,
            x: 1,
            y: 2,
            w: 1,
            h: 1,
            shape: "cylinder",
            radius: 1,
            height: 3,
            points: &[],
            block: 2,
            mode: "fill",
            from: None,
        },
    )
    .expect("圆柱填充失败");
    assert!(changed > 0);
    session::close(&id);
}

#[test]
fn test_edit_points_paints_only_listed_cells() {
    let id = session::create(sample_data());
    let points: Vec<[u32; 2]> = vec![[0, 0], [2, 3]];
    let (changed, _, _, _) = session::edit(
        &id,
        session::EditRequest {
            axis: "y",
            layer: 0,
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            shape: "points",
            radius: 0,
            height: 0,
            points: &points,
            block: 2,
            mode: "fill",
            from: None,
        },
    )
    .expect("画笔填充失败");
    assert!(changed <= 2, "实际改动 {}", changed);
    session::close(&id);
}

#[test]
fn test_edit_rejects_unknown_shape() {
    let id = session::create(sample_data());
    let result = session::edit(
        &id,
        session::EditRequest {
            axis: "y",
            layer: 0,
            x: 0,
            y: 0,
            w: 1,
            h: 1,
            shape: "torus",
            radius: 1,
            height: 1,
            points: &[],
            block: 2,
            mode: "fill",
            from: None,
        },
    );
    assert!(result.is_err());
    session::close(&id);
}

#[test]
fn test_block_entities_survive_sponge_roundtrip() {
    let mut data = sample_data();
    let mut fields = std::collections::HashMap::new();
    fields.insert(
        "id".to_string(),
        fastnbt::Value::String("minecraft:chest".to_string()),
    );
    data.block_entities.push(super::entities::BlockEntity {
        pos: [1, 0, 2],
        id: "minecraft:chest".to_string(),
        data: fields,
    });

    let path = temp_path("entities.schem");
    export(&data, "sponge_v2", &path).expect("导出失败");
    let reread = read_schematic(&path).expect("回读失败");
    assert_eq!(reread.block_entities.len(), 1);
    assert_eq!(reread.block_entities[0].pos, [1, 0, 2]);
    assert_eq!(reread.block_entities[0].id, "minecraft:chest");
}

#[test]
fn test_block_entities_survive_litematic_roundtrip() {
    let mut data = sample_data();
    data.block_entities.push(super::entities::BlockEntity {
        pos: [0, 1, 3],
        id: "minecraft:sign".to_string(),
        data: std::collections::HashMap::new(),
    });
    let path = temp_path("entities.litematic");
    export(&data, "litematic", &path).expect("导出失败");
    let reread = read_schematic(&path).expect("回读失败");
    assert_eq!(reread.block_entities.len(), 1);
    assert_eq!(reread.block_entities[0].pos, [0, 1, 3]);
    assert_eq!(reread.block_entities[0].id, "minecraft:sign");
}

#[test]
fn test_block_entities_survive_structure_roundtrip() {
    let mut data = sample_data();
    // 结构格式把实体挂在方块条目上，所以坐标必须落在非空气方块上（[0,0,1] 是木板）
    data.block_entities.push(super::entities::BlockEntity {
        pos: [0, 0, 1],
        id: "minecraft:furnace".to_string(),
        data: std::collections::HashMap::new(),
    });
    let path = temp_path("entities.nbt");
    export(&data, "structure", &path).expect("导出失败");

    // 先确认写出的字节里确实带了实体信息，便于区分写入侧与读取侧问题
    let raw = std::fs::read(&path).expect("读取导出文件失败");
    let raw_text = String::from_utf8_lossy(&raw);
    assert!(
        raw_text.contains("minecraft:furnace"),
        "导出字节中缺少实体 id，写入侧有问题"
    );
    let parsed: fastnbt::Value = fastnbt::from_bytes(&raw).expect("导出文件无法解析");
    let blocks = match &parsed {
        fastnbt::Value::Compound(map) => match map.get("blocks") {
            Some(fastnbt::Value::List(items)) => items.clone(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    let with_nbt = blocks
        .iter()
        .filter(|entry| match entry {
            fastnbt::Value::Compound(fields) => fields.contains_key("nbt"),
            _ => false,
        })
        .count();
    assert_eq!(with_nbt, 1, "blocks[].nbt 数量不符");

    let reread = read_schematic(&path).expect("回读失败");
    assert_eq!(reread.block_entities.len(), 1);
    assert_eq!(reread.block_entities[0].pos, [0, 0, 1]);
    assert_eq!(reread.block_entities[0].id, "minecraft:furnace");
}

#[test]
fn test_transform_moves_block_entities() {
    let mut data = sample_data();
    data.block_entities.push(super::entities::BlockEntity {
        pos: [0, 0, 0],
        id: "minecraft:chest".to_string(),
        data: std::collections::HashMap::new(),
    });
    let id = session::create(data);
    session::transform(&id, "rotate_cw").expect("旋转失败");
    session::inspect(&id, |s| {
        // 顺时针旋转后 (0,0,0) 落到新坐标 (0, 0, sx-1) = (0, 0, 2)
        assert_eq!(s.data.block_entities[0].pos, [0, 0, 2]);
        Ok(())
    })
    .expect("读取会话失败");
    session::close(&id);
}

#[test]
fn test_edit_prunes_replaced_block_entities() {
    let mut data = sample_data();
    data.block_entities.push(super::entities::BlockEntity {
        pos: [0, 0, 0],
        id: "minecraft:chest".to_string(),
        data: std::collections::HashMap::new(),
    });
    let id = session::create(data);
    session::edit(
        &id,
        session::EditRequest {
            axis: "y",
            layer: 0,
            x: 0,
            y: 0,
            w: 1,
            h: 1,
            shape: "rect",
            radius: 0,
            height: 0,
            points: &[],
            block: 2,
            mode: "fill",
            from: None,
        },
    )
    .expect("编辑失败");
    session::inspect(&id, |s| {
        assert!(s.data.block_entities.is_empty(), "被替换方块的实体应被移除");
        Ok(())
    })
    .expect("读取会话失败");
    session::close(&id);
}
