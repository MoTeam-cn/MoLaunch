//! 投影会话：打开后的方块数据常驻内存，按视口分片取用，并支持编辑与变换。
//!
//! 会话以 ID 引用，前端只按需取当前视口那一小块，因此不再受"一次性传输整份数据"
//! 的体积限制；同时保留编辑历史（撤销/重做）与变换能力。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use base64::Engine as _;
use once_cell::sync::Lazy;

use super::codec::{count_non_air, is_air};
use super::read::SchematicData;

/// 同时保留的会话数量上限（超出后淘汰最早的会话）
const MAX_SESSIONS: usize = 3;

/// 单次切片允许的最大格子数（防止前端一次索取过大）
const MAX_SLICE_CELLS: u32 = 1_048_576;

/// 一次编辑记录：索引 + 变更前/后的方块索引
struct EditRecord {
    indices: Vec<usize>,
    before: Vec<u16>,
    after: Vec<u16>,
}

/// 会话
pub struct Session {
    pub data: SchematicData,
    undo: Vec<EditRecord>,
    redo: Vec<EditRecord>,
    pub opened_at: u64,
}

struct Registry {
    sessions: HashMap<String, Session>,
    order: Vec<String>,
}

static REGISTRY: Lazy<Mutex<Registry>> = Lazy::new(|| {
    Mutex::new(Registry {
        sessions: HashMap::new(),
        order: Vec::new(),
    })
});

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// 新建会话，返回会话 ID
pub fn create(data: SchematicData) -> String {
    let serial = COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = format!("sch-{}-{}", std::process::id(), serial);
    let mut registry = match REGISTRY.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    registry.sessions.insert(
        id.clone(),
        Session {
            data,
            undo: Vec::new(),
            redo: Vec::new(),
            opened_at: serial,
        },
    );
    registry.order.push(id.clone());
    while registry.order.len() > MAX_SESSIONS {
        let evicted = registry.order.remove(0);
        registry.sessions.remove(&evicted);
    }
    id
}

/// 在会话上执行操作
fn with_session<R>(
    session_id: &str,
    action: impl FnOnce(&mut Session) -> Result<R, String>,
) -> Result<R, String> {
    let mut registry = match REGISTRY.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let session = registry
        .sessions
        .get_mut(session_id)
        .ok_or_else(|| "投影会话不存在或已过期，请重新打开文件".to_string())?;
    action(session)
}

/// 关闭会话
pub fn close(session_id: &str) {
    let mut registry = match REGISTRY.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    registry.sessions.remove(session_id);
    registry.order.retain(|id| id != session_id);
}

/// 清空所有会话（测试用）
pub fn clear_all() {
    let mut registry = match REGISTRY.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    registry.sessions.clear();
    registry.order.clear();
}

/// 三维索引（x 最快、其次 z、最后 y）
pub fn index_of(size: [u32; 3], x: u32, y: u32, z: u32) -> usize {
    (y as usize) * (size[2] as usize) * (size[0] as usize)
        + (z as usize) * (size[0] as usize)
        + (x as usize)
}

/// 切片平面尺寸：返回 (宽, 高, 层数)
pub fn plane_size(size: [u32; 3], axis: &str) -> Result<(u32, u32, u32), String> {
    match axis {
        "x" => Ok((size[2], size[1], size[0])),
        "z" => Ok((size[0], size[1], size[2])),
        "y" => Ok((size[0], size[2], size[1])),
        other => Err(format!("未知切片轴: {}", other)),
    }
}

/// 把平面坐标 + 层换算成三维坐标
fn to_xyz(size: [u32; 3], axis: &str, layer: u32, u: u32, v: u32) -> (u32, u32, u32) {
    match axis {
        "x" => (layer, v, u),
        "z" => (u, v, layer),
        _ => (u, layer, v),
    }
}

/// 取切片：返回 (宽, 高, base64 的 u16 小端数组)
pub fn slice(
    session_id: &str,
    axis: &str,
    layer: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<(u32, u32, String), String> {
    with_session(session_id, |session| {
        let size = session.data.size;
        let (plane_w, plane_h, layers) = plane_size(size, axis)?;
        if layer >= layers {
            return Err(format!("层号超出范围（0-{}）", layers.saturating_sub(1)));
        }
        let width = w.min(plane_w.saturating_sub(x)).max(1);
        let height = h.min(plane_h.saturating_sub(y)).max(1);
        if width * height > MAX_SLICE_CELLS {
            return Err("切片窗口过大，请缩小视口".to_string());
        }

        let mut cells: Vec<u16> = Vec::with_capacity((width * height) as usize);
        for row in 0..height {
            for col in 0..width {
                let (bx, by, bz) = to_xyz(size, axis, layer, x + col, y + row);
                let index = index_of(size, bx, by, bz);
                cells.push(session.data.blocks.get(index).copied().unwrap_or(0));
            }
        }
        Ok((width, height, encode_u16(&cells)))
    })
}

/// 在指定层的矩形区域填充方块
pub fn edit(
    session_id: &str,
    axis: &str,
    layer: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    block: u16,
    mode: &str,
    from: Option<u16>,
) -> Result<(u32, u64, bool, bool), String> {
    with_session(session_id, |session| {
        let size = session.data.size;
        let (plane_w, plane_h, layers) = plane_size(size, axis)?;
        if layer >= layers {
            return Err(format!("层号超出范围（0-{}）", layers.saturating_sub(1)));
        }
        if block as usize >= session.data.palette.len() {
            return Err("目标方块不在调色板中".to_string());
        }
        let width = w.min(plane_w.saturating_sub(x)).max(1);
        let height = h.min(plane_h.saturating_sub(y)).max(1);

        let mut indices: Vec<usize> = Vec::new();
        let mut before: Vec<u16> = Vec::new();
        let mut after: Vec<u16> = Vec::new();
        for row in 0..height {
            for col in 0..width {
                let (bx, by, bz) = to_xyz(size, axis, layer, x + col, y + row);
                let index = index_of(size, bx, by, bz);
                let current = session.data.blocks.get(index).copied().unwrap_or(0);
                if mode == "replace" {
                    match from {
                        Some(source) if current == source => {}
                        _ => continue,
                    }
                }
                if current == block {
                    continue;
                }
                indices.push(index);
                before.push(current);
                after.push(block);
            }
        }

        let changed = indices.len() as u32;
        if changed > 0 {
            for (offset, &index) in indices.iter().enumerate() {
                if let Some(slot) = session.data.blocks.get_mut(index) {
                    *slot = after[offset];
                }
            }
            session.undo.push(EditRecord {
                indices,
                before,
                after,
            });
            session.redo.clear();
        }
        let non_air = count_non_air(&session.data.blocks, &session.data.palette);
        Ok((
            changed,
            non_air,
            !session.undo.is_empty(),
            !session.redo.is_empty(),
        ))
    })
}

/// 撤销或重做
pub fn history(session_id: &str, action: &str) -> Result<(u32, u64, bool, bool), String> {
    with_session(session_id, |session| {
        let record = if action == "redo" {
            session.redo.pop()
        } else {
            session.undo.pop()
        };
        let Some(record) = record else {
            return Err("没有可回退的操作".to_string());
        };
        let values: Vec<u16> = if action == "redo" {
            record.after.clone()
        } else {
            record.before.clone()
        };
        for (offset, &index) in record.indices.iter().enumerate() {
            if let Some(slot) = session.data.blocks.get_mut(index) {
                *slot = values[offset];
            }
        }
        if action == "redo" {
            session.undo.push(record);
        } else {
            session.redo.push(record);
        }
        let non_air = count_non_air(&session.data.blocks, &session.data.palette);
        Ok((
            record.indices.len() as u32,
            non_air,
            !session.undo.is_empty(),
            !session.redo.is_empty(),
        ))
    })
}

/// 变换：旋转 / 镜像 / 上下翻转
pub fn transform(session_id: &str, kind: &str) -> Result<([u32; 3], u64), String> {
    with_session(session_id, |session| {
        let [sx, sy, sz] = session.data.size;
        let new_size: [u32; 3] = match kind {
            "rotate_cw" | "rotate_ccw" => [sz, sy, sx],
            "mirror_x" | "mirror_z" | "flip_y" => [sx, sy, sz],
            other => return Err(format!("未知变换: {}", other)),
        };
        let map: fn(u32, u32, u32, [u32; 3]) -> (u32, u32, u32) = match kind {
            "rotate_cw" => |x, y, z, [sx, _sy, _sz]| (z, y, sx - 1 - x),
            "rotate_ccw" => |x, y, z, [_sx, _sy, sz]| (sz - 1 - z, y, x),
            "mirror_x" => |x, y, z, [sx, _sy, _sz]| (sx - 1 - x, y, z),
            "mirror_z" => |x, y, z, [_sx, _sy, sz]| (x, y, sz - 1 - z),
            _ => |x, y, z, [_sx, sy, _sz]| (x, sy - 1 - y, z),
        };

        let mut blocks = vec![0u16; session.data.blocks.len()];
        for y in 0..sy {
            for z in 0..sz {
                for x in 0..sx {
                    let source = index_of([sx, sy, sz], x, y, z);
                    let value = session.data.blocks.get(source).copied().unwrap_or(0);
                    let (nx, ny, nz) = map(x, y, z, [sx, sy, sz]);
                    let target = index_of(new_size, nx, ny, nz);
                    if target < blocks.len() {
                        blocks[target] = value;
                    }
                }
            }
        }
        session.data.size = new_size;
        session.data.blocks = blocks;
        session.undo.clear();
        session.redo.clear();
        let non_air = count_non_air(&session.data.blocks, &session.data.palette);
        Ok((new_size, non_air))
    })
}

/// 供导出使用：在会话上执行只读操作
pub fn inspect<R>(
    session_id: &str,
    action: impl FnOnce(&Session) -> Result<R, String>,
) -> Result<R, String> {
    with_session(session_id, action)
}

/// 调色板中空气对应的索引（没有则返回 None）
pub fn air_index(palette: &[String]) -> Option<u16> {
    palette.iter().position(|id| is_air(id)).map(|v| v as u16)
}

fn encode_u16(values: &[u16]) -> String {
    let mut bytes = Vec::with_capacity(values.len() * 2);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
