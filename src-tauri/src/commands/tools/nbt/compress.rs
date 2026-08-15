//! gzip / zlib 压缩辅助（gzip 检测魔数 0x1f 0x8b）

use std::io::{Read, Write};

use crate::error_util::log_err;

/// gzip 解压（检测 gzip 魔数 0x1f 0x8b，如 player .dat / level.dat）
pub(super) fn gunzip_if_needed(raw: &[u8]) -> Result<Vec<u8>, String> {
    if raw.len() >= 2 && raw[0] == 0x1f && raw[1] == 0x8b {
        let mut decoder = flate2::read::GzDecoder::new(raw);
        let mut out = Vec::new();
        decoder
            .read_to_end(&mut out)
            .map_err(log_err("NBT gzip 解压失败"))?;
        Ok(out)
    } else {
        Ok(raw.to_vec())
    }
}

pub(super) fn gzip_compress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(data)
        .map_err(log_err("NBT gzip 压缩失败"))?;
    encoder.finish().map_err(log_err("NBT gzip 压缩失败"))
}

pub(super) fn zlib_compress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(data)
        .map_err(log_err("NBT zlib 压缩失败"))?;
    encoder.finish().map_err(log_err("NBT zlib 压缩失败"))
}

pub(super) fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(log_err("NBT zlib 解压失败"))?;
    Ok(out)
}
