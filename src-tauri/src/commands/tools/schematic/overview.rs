//! 投影概览：层密度与俯视高度图
//!
//! 逐层浏览时经常遇到"打开就落在空层"的困惑，也看不出整体形状。
//! 这里一次性算出两层信息：每层非空方块数（供层滑块定位与密度提示），
//! 以及俯视高度图（每列的最高非空方块与高度，供前端画等轴测立体预览）。
//!
//! 俯视高度图超过 `MAX_SIDE` 时按整数步长降采样，取块内最高者代表该区域，
//! 避免超大投影把几百万个格子全推给前端。

use base64::Engine as _;

use super::codec::is_air;
use super::session;

/// 俯视高度图单边最大格数
pub const MAX_SIDE: u32 = 256;

/// 俯视高度图里表示"该列全空"的方块索引
pub const EMPTY_INDEX: u16 = u16::MAX;

/// 概览结果
pub struct Overview {
    pub size: [u32; 3],
    /// 每层非空方块数，下标即层号（y 轴）
    pub layer_counts: Vec<u64>,
    /// 俯视高度图宽（可能因降采样小于实际尺寸）
    pub width: u32,
    pub height: u32,
    /// 降采样步长（1 表示未降采样）
    pub step: u32,
    /// 俯视每列最高方块的调色板索引（EMPTY_INDEX 表示全空）
    pub top_index: Vec<u16>,
    /// 俯视每列最高方块的高度（相对方块数，0 表示全空）
    pub top_height: Vec<u16>,
}

/// 计算概览
pub fn build(session_id: &str) -> Result<Overview, String> {
    session::inspect(session_id, |session| {
        let data = &session.data;
        let [sx, sy, sz] = data.size;
        let (sx_usize, sy_usize, sz_usize) = (sx as usize, sy as usize, sz as usize);

        // 每层非空方块数
        let mut layer_counts: Vec<u64> = vec![0; sy_usize];
        for (index, &palette_index) in data.blocks.iter().enumerate() {
            let id = data
                .palette
                .get(palette_index as usize)
                .map(String::as_str)
                .unwrap_or("");
            if is_air(id) {
                continue;
            }
            // index = x + z*sx + y*sx*sz
            let layer = index / (sx_usize * sz_usize).max(1);
            if layer < layer_counts.len() {
                layer_counts[layer] += 1;
            }
        }

        // 俯视降采样步长
        let longest = sx.max(sz);
        let step = if longest > MAX_SIDE {
            longest.div_ceil(MAX_SIDE)
        } else {
            1
        };
        let width = sx.div_ceil(step);
        let height = sz.div_ceil(step);

        let mut top_index: Vec<u16> = vec![EMPTY_INDEX; (width * height) as usize];
        let mut top_height: Vec<u16> = vec![0; (width * height) as usize];

        for y in 0..sy {
            for z in 0..sz {
                for x in 0..sx {
                    let index = session::index_of(data.size, x, y, z);
                    let palette_index = data.blocks.get(index).copied().unwrap_or(0);
                    let id = data
                        .palette
                        .get(palette_index as usize)
                        .map(String::as_str)
                        .unwrap_or("");
                    if is_air(id) {
                        continue;
                    }
                    let cell = (z / step) * width + (x / step);
                    let cell = cell as usize;
                    if cell >= top_index.len() {
                        continue;
                    }
                    // 逐层覆盖：后写入的层更高
                    if y + 1 >= u32::from(top_height[cell]) {
                        top_height[cell] = (y + 1).min(u16::MAX as u32) as u16;
                        top_index[cell] = palette_index;
                    }
                }
            }
        }

        Ok(Overview {
            size: data.size,
            layer_counts,
            width,
            height,
            step,
            top_index,
            top_height,
        })
    })
}

/// u16 数组 → base64（小端）
pub fn encode_u16(values: &[u16]) -> String {
    let mut bytes = Vec::with_capacity(values.len() * 2);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

#[cfg(test)]
#[path = "overview_test.rs"]
mod overview_test;
