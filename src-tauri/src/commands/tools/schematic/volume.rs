//! 体素数据：把整个体积的方块索引压成一条 zlib 流交给前端做 3D 渲染。
//!
//! 逐层切片只够 2D 浏览，3D 场景需要一次性拿到整体形状。方块索引是 u16，
//! 有地形的投影压缩后通常只剩原始体积的几个百分点，所以整份压一次传过去，
//! 不必再按块来回请求。

use std::io::Write;

use base64::Engine as _;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use super::codec::is_air;
use super::session;

/// 3D 预览允许的最大体积（方块数）。超过这个量压缩流和前端解包都会明显卡顿，
/// 逐层浏览仍然可用，因此直接拒绝而不是硬扛。
pub const MAX_PREVIEW_VOLUME: u64 = 16_000_000;

/// 体素数据
pub struct Volume {
    pub size: [u32; 3],
    pub palette: Vec<String>,
    /// zlib 流（带 zlib 头，对应浏览器的 deflate）的 base64
    pub blocks_base64: String,
    pub total: u64,
    pub non_air: u64,
    /// 空气在调色板中的下标（-1 表示调色板里没有空气）
    pub air_index: i32,
}

/// 打包整个体积
pub fn build(session_id: &str) -> Result<Volume, String> {
    session::inspect(session_id, |session| {
        let data = &session.data;
        let total = data.blocks.len() as u64;
        if total > MAX_PREVIEW_VOLUME {
            return Err(format!(
                "体积 {} 方块超过 3D 预览上限 {}，请改用逐层浏览",
                total, MAX_PREVIEW_VOLUME
            ));
        }

        let mut bytes: Vec<u8> = Vec::with_capacity(data.blocks.len() * 2);
        for value in &data.blocks {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&bytes)
            .map_err(|e| format!("压缩体素数据失败: {}", e))?;
        let compressed = encoder
            .finish()
            .map_err(|e| format!("压缩体素数据失败: {}", e))?;

        let non_air = data
            .blocks
            .iter()
            .filter(|&&index| {
                data.palette
                    .get(index as usize)
                    .map(|id| !is_air(id))
                    .unwrap_or(false)
            })
            .count() as u64;
        let air_index = data
            .palette
            .iter()
            .position(|id| is_air(id))
            .map(|index| index as i32)
            .unwrap_or(-1);

        Ok(Volume {
            size: data.size,
            palette: data.palette.clone(),
            blocks_base64: base64::engine::general_purpose::STANDARD.encode(&compressed),
            total,
            non_air,
            air_index,
        })
    })
}

#[cfg(test)]
#[path = "volume_test.rs"]
mod volume_test;
