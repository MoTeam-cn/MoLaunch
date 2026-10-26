//! 方块实体（箱子内容、告示牌文字、刷怪笼配置等）与实体的读取与回写。
//!
//! 三种格式对实体的描述方式不同：Sponge 用 `BlockEntities`（v2）/ `TileEntities`（v1），
//! Litematica 每个区域各有一份 `TileEntities`，原版结构文件把方块实体挂在 `blocks[].nbt` 上。
//! 这里统一归一化为「坐标 + id + 其余原始字段」，导出时再按目标格式还原，避免丢数据。

use std::collections::{HashMap, HashSet};

use fastnbt::Value;

/// 归一化后的方块实体
#[derive(Debug, Clone)]
pub struct BlockEntity {
    /// 绝对坐标（已换算到整体包围盒坐标系）
    pub pos: [i32; 3],
    pub id: String,
    /// 除坐标与 id 之外的原始字段
    pub data: HashMap<String, Value>,
}

/// 非方块实体（生物、掉落物等）：保留原始 NBT，导出时原样写回
#[derive(Debug, Clone)]
pub struct RawEntity {
    pub data: HashMap<String, Value>,
}

// ---------- 通用取值 ----------

fn compound(value: &Value) -> Option<&HashMap<String, Value>> {
    match value {
        Value::Compound(map) => Some(map),
        _ => None,
    }
}

fn as_int(value: Option<&Value>) -> Option<i32> {
    match value {
        Some(Value::Byte(v)) => Some(i32::from(*v)),
        Some(Value::Short(v)) => Some(i32::from(*v)),
        Some(Value::Int(v)) => Some(*v),
        Some(Value::Long(v)) => Some(*v as i32),
        _ => None,
    }
}

fn as_string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    }
}

fn as_list(value: Option<&Value>) -> Option<&Vec<Value>> {
    match value {
        Some(Value::List(items)) => Some(items),
        _ => None,
    }
}

/// 从 compound 中读取 [x, y, z] 形式的坐标（IntArray 或 Int 列表）
fn read_pos_array(map: &HashMap<String, Value>, key: &str) -> Option<[i32; 3]> {
    match map.get(key) {
        Some(Value::IntArray(array)) => {
            let values: Vec<i32> = array.iter().copied().collect();
            if values.len() >= 3 {
                Some([values[0], values[1], values[2]])
            } else {
                None
            }
        }
        Some(Value::List(items)) => {
            if items.len() < 3 {
                return None;
            }
            Some([
                as_int(items.first())?,
                as_int(items.get(1))?,
                as_int(items.get(2))?,
            ])
        }
        _ => None,
    }
}

/// 从 compound 中读取 {x, y, z} 形式的坐标
fn read_pos_compound(map: &HashMap<String, Value>, key: &str) -> Option<[i32; 3]> {
    let inner = compound(map.get(key)?)?;
    Some([
        as_int(inner.get("x"))?,
        as_int(inner.get("y"))?,
        as_int(inner.get("z"))?,
    ])
}

/// 把 id / pos 之外的字段抽出来
fn rest_fields(map: &HashMap<String, Value>, skip: &[&str]) -> HashMap<String, Value> {
    let mut out = HashMap::new();
    for (key, value) in map {
        if skip.iter().any(|item| item == key) {
            continue;
        }
        out.insert(key.clone(), value.clone());
    }
    out
}

// ---------- 读取 ----------

/// Sponge：`BlockEntities`（v2）或 `TileEntities`（v1）
pub fn read_sponge(root: &Value, offset: [i32; 3]) -> Vec<BlockEntity> {
    let Some(map) = compound(root) else {
        return Vec::new();
    };
    let list = as_list(map.get("BlockEntities"))
        .or_else(|| as_list(map.get("TileEntities")))
        .cloned()
        .unwrap_or_default();

    let mut out = Vec::new();
    for entry in list {
        let Some(fields) = compound(&entry) else {
            continue;
        };
        let pos = read_pos_array(fields, "Pos")
            .or_else(|| read_pos_array(fields, "pos"))
            .or_else(|| {
                Some([
                    as_int(fields.get("x"))?,
                    as_int(fields.get("y"))?,
                    as_int(fields.get("z"))?,
                ])
            });
        let Some(pos) = pos else {
            continue;
        };
        let id = as_string(fields.get("Id"))
            .or_else(|| as_string(fields.get("id")))
            .unwrap_or_else(|| "minecraft:chest".to_string());
        out.push(BlockEntity {
            pos: [pos[0] + offset[0], pos[1] + offset[1], pos[2] + offset[2]],
            id,
            data: rest_fields(fields, &["Pos", "pos", "Id", "id", "x", "y", "z"]),
        });
    }
    out
}

/// Litematica：区域内的 `TileEntities`
pub fn read_litematic(region: &Value, offset: [i32; 3]) -> Vec<BlockEntity> {
    let Some(map) = compound(region) else {
        return Vec::new();
    };
    let list = as_list(map.get("TileEntities"))
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for entry in list {
        let Some(fields) = compound(&entry) else {
            continue;
        };
        let Some(pos) = read_pos_compound(fields, "Pos") else {
            continue;
        };
        let id = as_string(fields.get("Id")).unwrap_or_else(|| "minecraft:chest".to_string());
        out.push(BlockEntity {
            pos: [pos[0] + offset[0], pos[1] + offset[1], pos[2] + offset[2]],
            id,
            data: rest_fields(fields, &["Pos", "Id"]),
        });
    }
    out
}

/// 原版结构文件：方块实体挂在 `blocks[].nbt` 上
pub fn read_structure(root: &Value) -> Vec<BlockEntity> {
    let Some(map) = compound(root) else {
        return Vec::new();
    };
    let Some(blocks) = as_list(map.get("blocks")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in blocks {
        let Some(fields) = compound(entry) else {
            continue;
        };
        let Some(nbt_value) = fields.get("nbt") else {
            continue;
        };
        let Some(nbt) = compound(nbt_value) else {
            continue;
        };
        let Some(pos) = read_pos_array(fields, "pos") else {
            continue;
        };
        let id = as_string(nbt.get("id"))
            .or_else(|| as_string(nbt.get("Id")))
            .unwrap_or_else(|| "minecraft:chest".to_string());
        out.push(BlockEntity {
            pos,
            id,
            data: rest_fields(nbt, &["id", "Id", "Pos", "pos", "x", "y", "z"]),
        });
    }
    out
}

/// 原版结构文件的非方块实体：`entities[]`
pub fn read_structure_entities(root: &Value) -> Vec<RawEntity> {
    let Some(map) = compound(root) else {
        return Vec::new();
    };
    as_list(map.get("entities"))
        .map(|items| {
            items
                .iter()
                .filter_map(compound)
                .map(|fields| RawEntity {
                    data: fields.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Litematica 区域内的非方块实体
pub fn read_litematic_entities(region: &Value) -> Vec<RawEntity> {
    let Some(map) = compound(region) else {
        return Vec::new();
    };
    as_list(map.get("Entities"))
        .map(|items| {
            items
                .iter()
                .filter_map(compound)
                .map(|fields| RawEntity {
                    data: fields.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

// ---------- 回写 ----------

/// Sponge v2：`BlockEntities`（Pos 为 IntArray）
pub fn to_sponge(entities: &[BlockEntity]) -> Vec<Value> {
    entities
        .iter()
        .map(|entity| {
            let mut fields: HashMap<String, Value> = entity.data.clone();
            fields.insert(
                "Pos".to_string(),
                Value::IntArray(fastnbt::IntArray::new(vec![
                    entity.pos[0],
                    entity.pos[1],
                    entity.pos[2],
                ])),
            );
            fields.insert("Id".to_string(), Value::String(entity.id.clone()));
            Value::Compound(fields)
        })
        .collect()
}

/// Litematica：`TileEntities`（Pos 为 {x,y,z}）
pub fn to_litematic(entities: &[BlockEntity]) -> Vec<Value> {
    entities
        .iter()
        .map(|entity| {
            let mut pos: HashMap<String, Value> = HashMap::new();
            pos.insert("x".to_string(), Value::Int(entity.pos[0]));
            pos.insert("y".to_string(), Value::Int(entity.pos[1]));
            pos.insert("z".to_string(), Value::Int(entity.pos[2]));
            let mut fields: HashMap<String, Value> = entity.data.clone();
            fields.insert("Pos".to_string(), Value::Compound(pos));
            fields.insert("Id".to_string(), Value::String(entity.id.clone()));
            Value::Compound(fields)
        })
        .collect()
}

/// 原版结构文件：把方块实体挂回对应方块的 `nbt` 字段
pub fn attach_to_structure_blocks(blocks: &mut [Value], entities: &[BlockEntity]) {
    let mut by_pos: HashMap<[i32; 3], &BlockEntity> = HashMap::new();
    for entity in entities {
        by_pos.insert(entity.pos, entity);
    }
    for entry in blocks.iter_mut() {
        let Value::Compound(fields) = entry else {
            continue;
        };
        let Some(pos) = read_pos_array(fields, "pos") else {
            continue;
        };
        let Some(entity) = by_pos.get(&pos) else {
            continue;
        };
        let mut nbt: HashMap<String, Value> = entity.data.clone();
        nbt.insert("id".to_string(), Value::String(entity.id.clone()));
        fields.insert("nbt".to_string(), Value::Compound(nbt));
    }
}

/// 非方块实体原样回写
pub fn to_raw(entities: &[RawEntity]) -> Vec<Value> {
    entities
        .iter()
        .map(|entity| Value::Compound(entity.data.clone()))
        .collect()
}

// ---------- 编辑与变换 ----------

/// 删除落在指定坐标集合上的方块实体（方块被替换后实体已失效）
pub fn prune(entities: &mut Vec<BlockEntity>, removed: &HashSet<[i32; 3]>) {
    if removed.is_empty() {
        return;
    }
    entities.retain(|entity| !removed.contains(&entity.pos));
}

/// 变换方块实体坐标（与方块使用同一套映射）
pub fn transform_positions(entities: &mut [BlockEntity], kind: &str, size: [u32; 3]) {
    let sx = size[0] as i32;
    let sy = size[1] as i32;
    let sz = size[2] as i32;
    for entity in entities.iter_mut() {
        let [x, y, z] = entity.pos;
        let (nx, ny, nz) = match kind {
            "rotate_cw" => (z, y, sx - 1 - x),
            "rotate_ccw" => (sz - 1 - z, y, x),
            "mirror_x" => (sx - 1 - x, y, z),
            "mirror_z" => (x, y, sz - 1 - z),
            "flip_y" => (x, sy - 1 - y, z),
            _ => (x, y, z),
        };
        entity.pos = [nx, ny, nz];
    }
}
