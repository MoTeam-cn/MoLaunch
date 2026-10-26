//! 投影概览测试：层密度、俯视高度图、降采样与错误分支。

use super::*;
use crate::commands::tools::schematic::read::SchematicData;
use crate::commands::tools::schematic::session;

/// 3×2×4，x 最快、其次 z、最后 y；1 与 2 为非空气方块
fn sample() -> SchematicData {
    SchematicData {
        format: "sponge_v2",
        name: "demo".to_string(),
        size: [3, 2, 4],
        palette: vec![
            "minecraft:air".to_string(),
            "minecraft:stone".to_string(),
            "minecraft:oak_planks".to_string(),
        ],
        blocks: vec![
            0, 1, 1, 2, 0, 1, 0, 0, 2, 2, 1, 0, 0, 2, 2, 1, 1, 0, 2, 0, 0, 1, 2, 2,
        ],
        block_entities: Vec::new(),
        raw_entities: Vec::new(),
    }
}

#[test]
fn test_layer_counts_ignore_air() {
    let id = session::create(sample());
    let data = build(&id).expect("概览失败");
    // y=0 有 7 个非空气方块，y=1 有 8 个
    assert_eq!(data.layer_counts, vec![7, 8]);
    session::close(&id);
}

#[test]
fn test_top_height_map_picks_highest_block() {
    let id = session::create(sample());
    let data = build(&id).expect("概览失败");
    assert_eq!(data.step, 1);
    assert_eq!((data.width, data.height), (3, 4));

    // (x=0, z=1)：y=0 与 y=1 都非空气，最高层为 y=1，相对高度 2
    let cell = 1 * data.width + 0;
    assert_eq!(data.top_height[cell as usize], 2);
    assert_eq!(data.top_index[cell as usize], 2);

    // (x=0, z=0) 两列都是空气
    assert_eq!(data.top_height[0], 0);
    assert_eq!(data.top_index[0], EMPTY_INDEX);
    session::close(&id);
}

#[test]
fn test_overview_rejects_unknown_session() {
    assert!(build("sch-not-exist").is_err());
}

#[test]
fn test_encode_u16_has_two_bytes_per_value() {
    let encoded = encode_u16(&[0, 1, 65535]);
    assert_eq!(encoded.len(), 8);
}

#[test]
fn test_large_plane_gets_downsampled() {
    // 300×1×300 超过单边 256，应当降采样
    let size = 300usize;
    let data = SchematicData {
        format: "sponge_v2",
        name: "big".to_string(),
        size: [size as u32, 1, size as u32],
        palette: vec!["minecraft:air".to_string(), "minecraft:stone".to_string()],
        blocks: vec![1; size * size],
        block_entities: Vec::new(),
        raw_entities: Vec::new(),
    };
    let id = session::create(data);
    let result = build(&id).expect("概览失败");
    assert!(result.step > 1, "步长应大于 1，实际 {}", result.step);
    assert!(result.width <= MAX_SIDE);
    assert!(result.height <= MAX_SIDE);
    assert_eq!(result.layer_counts, vec![(size * size) as u64]);
    session::close(&id);
}
