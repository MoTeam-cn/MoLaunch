//! 从存档 level.dat 提取种子（gzip+NBT 解析，返回十进制字符串）。

use std::io::Read;

use fastnbt::Value as NbtValue;

use crate::commands::tools::types::{ExtractSaveSeedParams, ExtractSaveSeedResult};
use crate::error_util::log_err;
use crate::log_info;
use crate::state::AppState;

use super::resolve_saves_dir;

/// 从存档 level.dat 提取种子
///
/// level.dat 是 gzip 压缩的 NBT 文件，根 compound 下 `Data` 子 compound 包含：
/// - `WorldGenSettings.seed`（Long，1.16+）
/// - `RandomSeed`（Long，1.15 及更早）
///
/// 优先读 WorldGenSettings.seed，回退 RandomSeed。返回十进制字符串
/// （MC 种子是 i64，JS Number 仅 53 位精度，无法精确表示）。
///
/// 路径安全：world_name 不允许含 ".." 或路径分隔符。
pub async fn extract_save_seed(
    state: &AppState,
    params: ExtractSaveSeedParams,
) -> Result<serde_json::Value, String> {
    if params.world_name.is_empty()
        || params.world_name.contains("..")
        || params.world_name.contains('/')
        || params.world_name.contains('\\')
    {
        return Err("非法存档名称".to_string());
    }

    let saves_dir = resolve_saves_dir(state, params.version_id.as_deref()).await;
    let level_dat = saves_dir.join(&params.world_name).join("level.dat");
    if !level_dat.is_file() {
        return Err(format!("level.dat 不存在: {}", level_dat.display()));
    }

    log_info!("[Archive] 提取种子: {}", level_dat.display());

    let level_dat_clone = level_dat.clone();
    let (seed, source) =
        tokio::task::spawn_blocking(move || -> Result<(String, String), String> {
            let raw = std::fs::read(&level_dat_clone).map_err(log_err("读取 level.dat 失败"))?;
            if raw.len() < 2 {
                return Err("level.dat 文件过小".to_string());
            }
            // gzip 解压（level.dat 固定 gzip 压缩）
            let data: Vec<u8> = if raw[0] == 0x1f && raw[1] == 0x8b {
                let mut decoder = flate2::read::GzDecoder::new(&raw[..]);
                let mut out = Vec::new();
                decoder
                    .read_to_end(&mut out)
                    .map_err(log_err("level.dat gzip 解压失败"))?;
                out
            } else {
                raw
            };

            let root: NbtValue =
                fastnbt::from_bytes(&data).map_err(|e| format!("level.dat NBT 解析失败: {}", e))?;

            // 根 compound → Data → 优先 WorldGenSettings.seed，回退 RandomSeed
            let data = match &root {
                NbtValue::Compound(c) => match c.get("Data") {
                    Some(NbtValue::Compound(d)) => d,
                    _ => return Err("level.dat 缺少 Data 字段".to_string()),
                },
                _ => return Err("level.dat 根非 Compound".to_string()),
            };

            // 优先 WorldGenSettings.seed（1.16+）
            if let Some(NbtValue::Compound(wgs)) = data.get("WorldGenSettings") {
                if let Some(NbtValue::Long(n)) = wgs.get("seed") {
                    return Ok((n.to_string(), "WorldGenSettings.seed".to_string()));
                }
            }
            // 回退 RandomSeed（1.15 及更早）
            if let Some(NbtValue::Long(n)) = data.get("RandomSeed") {
                return Ok((n.to_string(), "RandomSeed".to_string()));
            }
            Err("level.dat 未找到 WorldGenSettings.seed 或 RandomSeed".to_string())
        })
        .await
        .map_err(log_err("提取种子任务失败"))??;

    log_info!("[Archive] 种子提取成功: {} (来源: {})", seed, source);
    let result = ExtractSaveSeedResult { seed, source };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}
