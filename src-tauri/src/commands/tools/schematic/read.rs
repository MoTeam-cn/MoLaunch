//! 投影文件读取：Sponge v1/v2、Litematica、原版结构文件。
//!
//! 三种格式都基于公开规范实现：方块坐标统一按 x 最快、其次 z、最后 y 排列，
//! 调色板统一压缩为连续索引，便于前端一次性取回后按层切片。

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use fastnbt::Value;

use super::codec::{
    count_non_air, decode_packed_longs, decode_varints, join_block_state, normalize_block_id,
    normalize_block_state, MAX_VOLUME,
};

/// 解析后的投影数据
pub struct SchematicData {
    pub format: &'static str,
    pub name: String,
    pub size: [u32; 3],
    pub palette: Vec<String>,
    pub blocks: Vec<u16>,
    /// 方块实体（箱子内容、告示牌文字等），随编辑与导出保留
    pub block_entities: Vec<super::entities::BlockEntity>,
    /// 非方块实体（生物等），仅在未做变换时原样保留
    pub raw_entities: Vec<super::entities::RawEntity>,
}

/// 读取投影文件，自动识别格式
pub fn read_schematic(path: &Path) -> Result<SchematicData, String> {
    let root = load_root(path)?;
    let name = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "schematic".to_string());

    let data = if get(&root, "Regions").is_some() {
        read_litematic(&root, &name)?
    } else if get(&root, "blocks").is_some() && get(&root, "palette").is_some() {
        read_structure(&root, &name)?
    } else if get(&root, "BlockData").is_some() && get(&root, "Palette").is_some() {
        read_sponge(&root, &name)?
    } else if get(&root, "Blocks").is_some() && get(&root, "Data").is_some() {
        read_mcedit_legacy(&root, &name)?
    } else {
        return Err(
            "无法识别的投影格式（支持 Sponge .schematic、Litematica .litematic、原版结构 .nbt）"
                .to_string(),
        );
    };

    let volume = u64::from(data.size[0]) * u64::from(data.size[1]) * u64::from(data.size[2]);
    if volume > MAX_VOLUME {
        return Err(format!(
            "投影体积过大（{} 个方块，上限 {}），请先裁剪后再打开",
            volume, MAX_VOLUME
        ));
    }
    if data.blocks.len() as u64 != volume {
        return Err("投影方块数据长度与尺寸不匹配".to_string());
    }
    Ok(data)
}

/// 非空气方块数量（供上层统计）
pub fn non_air_count(data: &SchematicData) -> u64 {
    count_non_air(&data.blocks, &data.palette)
}

fn load_root(path: &Path) -> Result<Value, String> {
    let raw = std::fs::read(path).map_err(|e| format!("读取投影文件失败: {}", e))?;
    let bytes = if raw.len() > 2 && raw[0] == 0x1f && raw[1] == 0x8b {
        let mut decoder = flate2::read::GzDecoder::new(raw.as_slice());
        let mut out = Vec::new();
        decoder
            .read_to_end(&mut out)
            .map_err(|e| format!("gzip 解压失败: {}", e))?;
        out
    } else {
        raw
    };
    fastnbt::from_bytes::<Value>(&bytes).map_err(|e| format!("NBT 解析失败: {}", e))
}

// ---------- Sponge（v1 / v2 共用主体） ----------

fn read_sponge(root: &Value, name: &str) -> Result<SchematicData, String> {
    let width = as_int(get(root, "Width")).unwrap_or(0);
    let height = as_int(get(root, "Height")).unwrap_or(0);
    let length = as_int(get(root, "Length")).unwrap_or(0);
    if width <= 0 || height <= 0 || length <= 0 {
        return Err("投影尺寸非法".to_string());
    }

    let palette_compound = compound(get(root, "Palette")).ok_or("缺少 Palette 调色板")?;
    let mut max_index: i64 = -1;
    let mut entries: Vec<(i64, String)> = Vec::with_capacity(palette_compound.len());
    for (id, value) in palette_compound {
        let index = as_int(Some(value)).ok_or("调色板索引非法")?;
        if index > max_index {
            max_index = index;
        }
        // Sponge 的 Palette 键本身就是完整 blockstate（含 [props]），原样保留
        entries.push((index, normalize_block_state(id)));
    }
    if max_index < 0 {
        return Err("调色板为空".to_string());
    }

    let mut palette: Vec<String> = vec!["minecraft:air".to_string(); (max_index + 1) as usize];
    for (index, id) in entries {
        palette[index as usize] = id;
    }

    let block_data = byte_array(get(root, "BlockData")).ok_or("缺少 BlockData")?;
    let indices = decode_varints(&block_data)?;
    let expected = (width as usize) * (height as usize) * (length as usize);
    if indices.len() != expected {
        return Err(format!(
            "方块数据长度不符（期望 {}，实际 {}）",
            expected,
            indices.len()
        ));
    }

    let blocks = to_blocks(&indices, palette.len())?;
    let format = if get(root, "Schematic").is_some() {
        "sponge_v2"
    } else {
        "sponge_v1"
    };
    Ok(SchematicData {
        format,
        name: name.to_string(),
        size: [width as u32, height as u32, length as u32],
        palette,
        blocks,
        block_entities: super::entities::read_sponge(root, [0, 0, 0]),
        raw_entities: Vec::new(),
    })
}

// ---------- Litematica ----------

fn read_litematic(root: &Value, name: &str) -> Result<SchematicData, String> {
    let regions = compound(get(root, "Regions")).ok_or("缺少 Regions")?;
    if regions.is_empty() {
        return Err("投影没有任何区域".to_string());
    }

    // 先算全局包围盒（Size 可能为负，表示向负方向延伸）
    let mut min = [i64::MAX; 3];
    let mut max = [i64::MIN; 3];
    for value in regions.values() {
        let position = compound(get(value, "Position")).ok_or("区域缺少 Position")?;
        let size = compound(get(value, "Size")).ok_or("区域缺少 Size")?;
        let pos = [
            as_int(position.get("x")).unwrap_or(0),
            as_int(position.get("y")).unwrap_or(0),
            as_int(position.get("z")).unwrap_or(0),
        ];
        let dim = [
            as_int(size.get("x")).unwrap_or(0),
            as_int(size.get("y")).unwrap_or(0),
            as_int(size.get("z")).unwrap_or(0),
        ];
        for axis in 0..3 {
            let a = pos[axis];
            let b = pos[axis] + dim[axis];
            min[axis] = min[axis].min(a.min(b));
            max[axis] = max[axis].max(a.max(b));
        }
    }

    let sx = (max[0] - min[0]).max(0) as u32;
    let sy = (max[1] - min[1]).max(0) as u32;
    let sz = (max[2] - min[2]).max(0) as u32;
    if sx == 0 || sy == 0 || sz == 0 {
        return Err("投影尺寸非法".to_string());
    }

    let mut palette: Vec<String> = Vec::new();
    let mut blocks: Vec<u16> = vec![0; (sx as usize) * (sy as usize) * (sz as usize)];

    for value in regions.values() {
        let position = compound(get(value, "Position")).ok_or("区域缺少 Position")?;
        let size = compound(get(value, "Size")).ok_or("区域缺少 Size")?;
        let pos = [
            as_int(position.get("x")).unwrap_or(0),
            as_int(position.get("y")).unwrap_or(0),
            as_int(position.get("z")).unwrap_or(0),
        ];
        let dim = [
            as_int(size.get("x")).unwrap_or(0).abs(),
            as_int(size.get("y")).unwrap_or(0).abs(),
            as_int(size.get("z")).unwrap_or(0).abs(),
        ];
        let offset = [pos[0] - min[0], pos[1] - min[1], pos[2] - min[2]];

        let local_palette =
            list(get(value, "BlockStatePalette")).ok_or("区域缺少 BlockStatePalette")?;
        let mut index_map: Vec<u16> = Vec::with_capacity(local_palette.len());
        for entry in local_palette {
            let id = block_state_from_entry(entry);
            let global = match palette.iter().position(|existing| existing == &id) {
                Some(found) => found as u16,
                None => {
                    palette.push(id);
                    (palette.len() - 1) as u16
                }
            };
            index_map.push(global);
        }

        let states = long_array(get(value, "BlockStates")).unwrap_or_default();
        let bits = super::codec::bits_per_entry(local_palette.len());
        let local_volume = (dim[0] as usize) * (dim[1] as usize) * (dim[2] as usize);
        let decoded = decode_packed_longs(&states, bits, local_volume);

        for y in 0..dim[1] as usize {
            for z in 0..dim[2] as usize {
                for x in 0..dim[0] as usize {
                    let local_index = (y * dim[2] as usize + z) * dim[0] as usize + x;
                    let Some(&raw_index) = decoded.get(local_index) else {
                        continue;
                    };
                    let Some(&global_index) = index_map.get(raw_index as usize) else {
                        continue;
                    };
                    let gx = offset[0] + x as i64;
                    let gy = offset[1] + y as i64;
                    let gz = offset[2] + z as i64;
                    if gx < 0 || gy < 0 || gz < 0 {
                        continue;
                    }
                    if gx as u32 >= sx || gy as u32 >= sy || gz as u32 >= sz {
                        continue;
                    }
                    let target =
                        ((gy as usize) * sz as usize + gz as usize) * sx as usize + gx as usize;
                    if target < blocks.len() {
                        blocks[target] = global_index;
                    }
                }
            }
        }
    }

    if palette.is_empty() {
        return Err("投影调色板为空".to_string());
    }

    // 收集各区域的方块实体（坐标已按包围盒偏移换算；read.rs 的 as_int 为 i64）
    let mut block_entities: Vec<super::entities::BlockEntity> = Vec::new();
    let mut raw_entities: Vec<super::entities::RawEntity> = Vec::new();
    for value in regions.values() {
        let position = compound(get(value, "Position")).ok_or("区域缺少 Position")?;
        let offset: [i32; 3] = [
            (as_int(position.get("x")).unwrap_or(0) - min[0]) as i32,
            (as_int(position.get("y")).unwrap_or(0) - min[1]) as i32,
            (as_int(position.get("z")).unwrap_or(0) - min[2]) as i32,
        ];
        block_entities.extend(super::entities::read_litematic(value, offset));
        raw_entities.extend(super::entities::read_litematic_entities(value));
    }

    Ok(SchematicData {
        format: "litematic",
        name: name.to_string(),
        size: [sx, sy, sz],
        palette,
        blocks,
        block_entities,
        raw_entities,
    })
}

// ---------- 原版结构文件 ----------

fn read_structure(root: &Value, name: &str) -> Result<SchematicData, String> {
    let size_list = list(get(root, "size")).ok_or("缺少 size")?;
    if size_list.len() != 3 {
        return Err("size 字段非法".to_string());
    }
    let sx = as_int(size_list.first()).unwrap_or(0);
    let sy = as_int(size_list.get(1)).unwrap_or(0);
    let sz = as_int(size_list.get(2)).unwrap_or(0);
    if sx <= 0 || sy <= 0 || sz <= 0 {
        return Err("投影尺寸非法".to_string());
    }

    let palette_list = list(get(root, "palette")).ok_or("缺少 palette")?;
    let mut palette: Vec<String> = Vec::with_capacity(palette_list.len());
    for entry in palette_list {
        palette.push(block_state_from_entry(entry));
    }

    let mut blocks: Vec<u16> = vec![0; (sx as usize) * (sy as usize) * (sz as usize)];
    if let Some(block_list) = list(get(root, "blocks")) {
        for entry in block_list {
            let Some(pos) = list(get(entry, "pos")) else {
                continue;
            };
            if pos.len() != 3 {
                continue;
            }
            let x = as_int(pos.first()).unwrap_or(-1);
            let y = as_int(pos.get(1)).unwrap_or(-1);
            let z = as_int(pos.get(2)).unwrap_or(-1);
            if x < 0 || y < 0 || z < 0 {
                continue;
            }
            let state = as_int(get(entry, "state")).unwrap_or(0);
            let target = ((y as usize) * sz as usize + z as usize) * sx as usize + x as usize;
            if target < blocks.len() {
                blocks[target] = state.clamp(0, u16::MAX as i64) as u16;
            }
        }
    }

    Ok(SchematicData {
        format: "structure",
        name: name.to_string(),
        size: [sx as u32, sy as u32, sz as u32],
        palette,
        blocks,
        block_entities: super::entities::read_structure(root),
        raw_entities: super::entities::read_structure_entities(root),
    })
}

// ---------- 旧版 MCEdit（数字 id + 元数据） ----------

/// 解析 1.13 之前的 MCEdit 投影：`Blocks` 存方块 id，`Data` 存元数据，
/// `AddBlocks`（可选）以半字节形式存高位 id。
fn read_mcedit_legacy(root: &Value, name: &str) -> Result<SchematicData, String> {
    let width = as_int(get(root, "Width")).unwrap_or(0);
    let height = as_int(get(root, "Height")).unwrap_or(0);
    let length = as_int(get(root, "Length")).unwrap_or(0);
    if width <= 0 || height <= 0 || length <= 0 {
        return Err("投影尺寸非法".to_string());
    }

    let blocks_raw = byte_array(get(root, "Blocks")).ok_or("缺少 Blocks")?;
    let data_raw = byte_array(get(root, "Data")).unwrap_or_default();
    let add_raw = byte_array(get(root, "AddBlocks")).unwrap_or_default();

    let volume = (width as usize) * (height as usize) * (length as usize);
    if blocks_raw.len() < volume {
        return Err(format!(
            "Blocks 长度不足（期望 {}，实际 {}）",
            volume,
            blocks_raw.len()
        ));
    }

    let mut palette: Vec<String> = Vec::new();
    // (id, meta) 只有几十种组合，缓存后避免逐方块重复格式化字符串
    let mut legacy_index: HashMap<(u16, u8), u16> = HashMap::new();
    let mut blocks: Vec<u16> = Vec::with_capacity(volume);

    for position in 0..volume {
        let base = u16::from(blocks_raw[position]);
        // AddBlocks 每字节存两个方块的高位半字节
        let high = if add_raw.is_empty() {
            0u16
        } else {
            let byte = add_raw[position / 2];
            if position % 2 == 0 {
                u16::from(byte & 0x0f)
            } else {
                u16::from((byte >> 4) & 0x0f)
            }
        };
        let id = base | (high << 8);
        let meta = data_raw.get(position).copied().unwrap_or(0) & 0x0f;
        let index = match legacy_index.get(&(id, meta)) {
            Some(&found) => found,
            None => {
                if palette.len() > u16::MAX as usize {
                    return Err("调色板条目过多".to_string());
                }
                let name_id = super::legacy_blocks::legacy_block_name(id, meta)
                    .unwrap_or_else(|| format!("minecraft:legacy_{}_{}", id, meta));
                palette.push(name_id);
                let assigned = (palette.len() - 1) as u16;
                legacy_index.insert((id, meta), assigned);
                assigned
            }
        };
        blocks.push(index);
    }

    Ok(SchematicData {
        format: "mcedit_legacy",
        name: name.to_string(),
        size: [width as u32, height as u32, length as u32],
        palette,
        blocks,
        // 旧版格式的 TileEntities 只有告示牌等少数类型，坐标已是 0 基
        block_entities: super::entities::read_legacy(root),
        raw_entities: Vec::new(),
    })
}

/// 从 `{Name, Properties}` 形式的调色板条目还原完整 blockstate。
///
/// 1.13 之后方块名与属性分离存储，属性直接丢会丢掉朝向、上下半砖、水logged 等
/// 影响外观的信息，导出时形状就变了。
fn block_state_from_entry(entry: &Value) -> String {
    let name = get(entry, "Name")
        .and_then(as_string)
        .unwrap_or_else(|| "minecraft:air".to_string());
    let base = normalize_block_id(&name);
    let mut props: Vec<(String, String)> = Vec::new();
    if let Some(map) = compound(get(entry, "Properties")) {
        for (key, value) in map {
            if let Some(text) = as_string(value) {
                props.push((key.clone(), text));
            }
        }
    }
    props.sort_by(|a, b| a.0.cmp(&b.0));
    join_block_state(&base, &props)
}

// ---------- NBT 取值helper ----------

fn to_blocks(indices: &[u32], palette_len: usize) -> Result<Vec<u16>, String> {
    if palette_len > u16::MAX as usize {
        return Err("调色板条目过多（超过 65535）".to_string());
    }
    let mut out = Vec::with_capacity(indices.len());
    for &index in indices {
        if index as usize >= palette_len {
            return Err("方块索引超出调色板范围".to_string());
        }
        out.push(index as u16);
    }
    Ok(out)
}

fn compound(value: Option<&Value>) -> Option<&HashMap<String, Value>> {
    match value {
        Some(Value::Compound(map)) => Some(map),
        _ => None,
    }
}

fn get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    compound(Some(value)).and_then(|map| map.get(key))
}

fn as_int(value: Option<&Value>) -> Option<i64> {
    match value {
        Some(Value::Byte(v)) => Some(i64::from(*v)),
        Some(Value::Short(v)) => Some(i64::from(*v)),
        Some(Value::Int(v)) => Some(i64::from(*v)),
        Some(Value::Long(v)) => Some(*v),
        _ => None,
    }
}

fn as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        _ => None,
    }
}

fn byte_array(value: Option<&Value>) -> Option<Vec<u8>> {
    match value {
        Some(Value::ByteArray(array)) => Some(array.iter().map(|&b| b as u8).collect()),
        _ => None,
    }
}

fn long_array(value: Option<&Value>) -> Option<Vec<i64>> {
    match value {
        Some(Value::LongArray(array)) => Some(array.iter().copied().collect()),
        _ => None,
    }
}

fn list(value: Option<&Value>) -> Option<&Vec<Value>> {
    match value {
        Some(Value::List(items)) => Some(items),
        _ => None,
    }
}
