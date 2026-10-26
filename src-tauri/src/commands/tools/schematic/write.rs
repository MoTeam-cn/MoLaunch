//! 投影导出：Sponge v2 / Litematica / 原版结构文件。
//!
//! 三种格式均按公开规范生成，只写必需字段，保证能被游戏与主流投影工具读取。

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use fastnbt::{SerOpts, Value};

use super::codec::{bits_per_entry, encode_packed_longs, encode_varints};
use super::read::SchematicData;

/// 导出目标格式
pub fn export(data: &SchematicData, format: &str, target: &Path) -> Result<u64, String> {
    let bytes = match format {
        "sponge_v2" => gzip(&build_sponge(data)?)?,
        "litematic" => gzip(&build_litematic(data)?)?,
        "structure" => build_structure(data)?,
        other => return Err(format!("不支持的导出格式: {}", other)),
    };
    std::fs::write(target, &bytes).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(bytes.len() as u64)
}

fn gzip(payload: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(payload)
        .map_err(|e| format!("压缩失败: {}", e))?;
    encoder.finish().map_err(|e| format!("压缩失败: {}", e))
}

fn serialize(value: &Value) -> Result<Vec<u8>, String> {
    fastnbt::to_bytes_with_opts(value, SerOpts::new().root_name(""))
        .map_err(|e| format!("NBT 序列化失败: {}", e))
}

fn build_sponge(data: &SchematicData) -> Result<Vec<u8>, String> {
    let mut palette_map: HashMap<String, Value> = HashMap::new();
    for (index, id) in data.palette.iter().enumerate() {
        palette_map.insert(id.clone(), Value::Int(index as i32));
    }
    let block_data = encode_varints(
        &data
            .blocks
            .iter()
            .map(|&v| u32::from(v))
            .collect::<Vec<_>>(),
    );

    let mut metadata: HashMap<String, Value> = HashMap::new();
    metadata.insert("Name".to_string(), Value::String(data.name.clone()));
    metadata.insert("Author".to_string(), Value::String("MoLaunch".to_string()));

    let mut root: HashMap<String, Value> = HashMap::new();
    root.insert("Schematic".to_string(), Value::Int(2));
    root.insert("Version".to_string(), Value::Int(2));
    root.insert("DataVersion".to_string(), Value::Int(3953));
    root.insert("Width".to_string(), Value::Short(data.size[0] as i16));
    root.insert("Height".to_string(), Value::Short(data.size[1] as i16));
    root.insert("Length".to_string(), Value::Short(data.size[2] as i16));
    root.insert(
        "PaletteMax".to_string(),
        Value::Int(data.palette.len() as i32),
    );
    root.insert("Palette".to_string(), Value::Compound(palette_map));
    root.insert(
        "BlockData".to_string(),
        Value::ByteArray(fastnbt::ByteArray::new(
            block_data.iter().map(|&b| b as i8).collect(),
        )),
    );
    root.insert("Metadata".to_string(), Value::Compound(metadata));
    root.insert(
        "BlockEntities".to_string(),
        Value::List(super::entities::to_sponge(&data.block_entities)),
    );
    serialize(&Value::Compound(root))
}

fn build_litematic(data: &SchematicData) -> Result<Vec<u8>, String> {
    let bits = bits_per_entry(data.palette.len());
    let states = encode_packed_longs(
        &data
            .blocks
            .iter()
            .map(|&v| u32::from(v))
            .collect::<Vec<_>>(),
        bits,
    );

    let mut palette_list: Vec<Value> = Vec::with_capacity(data.palette.len());
    for id in &data.palette {
        let mut entry: HashMap<String, Value> = HashMap::new();
        entry.insert("Name".to_string(), Value::String(id.clone()));
        entry.insert("Properties".to_string(), Value::Compound(HashMap::new()));
        palette_list.push(Value::Compound(entry));
    }

    let mut position: HashMap<String, Value> = HashMap::new();
    position.insert("x".to_string(), Value::Int(0));
    position.insert("y".to_string(), Value::Int(0));
    position.insert("z".to_string(), Value::Int(0));

    let mut size: HashMap<String, Value> = HashMap::new();
    size.insert("x".to_string(), Value::Int(data.size[0] as i32));
    size.insert("y".to_string(), Value::Int(data.size[1] as i32));
    size.insert("z".to_string(), Value::Int(data.size[2] as i32));

    let mut region: HashMap<String, Value> = HashMap::new();
    region.insert("Position".to_string(), Value::Compound(position));
    region.insert("Size".to_string(), Value::Compound(size));
    region.insert("BlockStatePalette".to_string(), Value::List(palette_list));
    region.insert(
        "BlockStates".to_string(),
        Value::LongArray(fastnbt::LongArray::new(states)),
    );
    region.insert(
        "TileEntities".to_string(),
        Value::List(super::entities::to_litematic(&data.block_entities)),
    );
    region.insert(
        "Entities".to_string(),
        Value::List(super::entities::to_raw(&data.raw_entities)),
    );
    region.insert("PendingBlockTicks".to_string(), Value::List(Vec::new()));
    region.insert("PendingFluidTicks".to_string(), Value::List(Vec::new()));

    let mut regions: HashMap<String, Value> = HashMap::new();
    regions.insert("MoLaunch".to_string(), Value::Compound(region));

    let mut enclosing: HashMap<String, Value> = HashMap::new();
    enclosing.insert("x".to_string(), Value::Int(data.size[0] as i32));
    enclosing.insert("y".to_string(), Value::Int(data.size[1] as i32));
    enclosing.insert("z".to_string(), Value::Int(data.size[2] as i32));

    let mut metadata: HashMap<String, Value> = HashMap::new();
    metadata.insert("Name".to_string(), Value::String(data.name.clone()));
    metadata.insert("Author".to_string(), Value::String("MoLaunch".to_string()));
    metadata.insert("Description".to_string(), Value::String(String::new()));
    metadata.insert("RegionCount".to_string(), Value::Int(1));
    metadata.insert(
        "TotalBlocks".to_string(),
        Value::Int(data.blocks.len() as i32),
    );
    metadata.insert(
        "TotalVolume".to_string(),
        Value::Int(data.blocks.len() as i32),
    );
    metadata.insert("EnclosingSize".to_string(), Value::Compound(enclosing));
    metadata.insert(
        "TimeCreated".to_string(),
        Value::Long(chrono::Local::now().timestamp_millis()),
    );
    metadata.insert(
        "TimeModified".to_string(),
        Value::Long(chrono::Local::now().timestamp_millis()),
    );

    let mut root: HashMap<String, Value> = HashMap::new();
    root.insert("MinecraftDataVersion".to_string(), Value::Int(3953));
    root.insert("Version".to_string(), Value::Int(6));
    root.insert("Metadata".to_string(), Value::Compound(metadata));
    root.insert("Regions".to_string(), Value::Compound(regions));
    serialize(&Value::Compound(root))
}

fn build_structure(data: &SchematicData) -> Result<Vec<u8>, String> {
    let size_list = vec![
        Value::Int(data.size[0] as i32),
        Value::Int(data.size[1] as i32),
        Value::Int(data.size[2] as i32),
    ];

    let mut palette_list: Vec<Value> = Vec::with_capacity(data.palette.len());
    for id in &data.palette {
        let mut entry: HashMap<String, Value> = HashMap::new();
        entry.insert("Name".to_string(), Value::String(id.clone()));
        palette_list.push(Value::Compound(entry));
    }

    let mut blocks_list: Vec<Value> = Vec::new();
    let sx = data.size[0] as usize;
    let sz = data.size[2] as usize;
    for (index, &state) in data.blocks.iter().enumerate() {
        let id = data
            .palette
            .get(state as usize)
            .map(String::as_str)
            .unwrap_or("");
        if super::codec::is_air(id) {
            continue;
        }
        let x = index % sx;
        let z = (index / sx) % sz;
        let y = index / (sx * sz);
        let mut entry: HashMap<String, Value> = HashMap::new();
        entry.insert(
            "pos".to_string(),
            Value::List(vec![
                Value::Int(x as i32),
                Value::Int(y as i32),
                Value::Int(z as i32),
            ]),
        );
        entry.insert("state".to_string(), Value::Int(i32::from(state)));
        blocks_list.push(Value::Compound(entry));
    }

    super::entities::attach_to_structure_blocks(&mut blocks_list, &data.block_entities);

    let mut root: HashMap<String, Value> = HashMap::new();
    root.insert("size".to_string(), Value::List(size_list));
    root.insert("palette".to_string(), Value::List(palette_list));
    root.insert("blocks".to_string(), Value::List(blocks_list));
    root.insert(
        "entities".to_string(),
        Value::List(super::entities::to_raw(&data.raw_entities)),
    );
    root.insert("DataVersion".to_string(), Value::Int(3953));
    serialize(&Value::Compound(root))
}
