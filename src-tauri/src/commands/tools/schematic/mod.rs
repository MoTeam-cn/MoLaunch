//! 投影工具（只读预览 + 格式导出）
//!
//! 支持 Sponge .schematic（v1/v2）、Litematica .litematic、原版结构 .nbt。
//! 三种格式均按公开规范自行实现解析与写出，不依赖第三方投影库。

mod codec;
mod read;
mod write;

#[cfg(test)]
mod schematic_test;

use std::path::PathBuf;

use crate::commands::tools::types::{
    SchematicBlockCount, SchematicExportParams, SchematicExportResult, SchematicOpenParams,
    SchematicOpenResult,
};

/// 单次返回的方块统计条目上限
const TOP_COUNT_LIMIT: usize = 30;

/// 打开投影文件，返回调色板 + 压缩后的方块索引
pub async fn open(params: SchematicOpenParams) -> Result<SchematicOpenResult, String> {
    let path = PathBuf::from(&params.path);
    if !path.is_file() {
        return Err("投影文件不存在".to_string());
    }
    tokio::task::spawn_blocking(move || {
        let data = read::read_schematic(&path)?;
        let counts = codec::top_counts(&data.blocks, &data.palette, TOP_COUNT_LIMIT)
            .into_iter()
            .map(|(id, count)| SchematicBlockCount { id, count })
            .collect();
        let non_air = read::non_air_count(&data);
        Ok(SchematicOpenResult {
            format: data.format.to_string(),
            name: data.name.clone(),
            size: data.size,
            palette: data.palette.clone(),
            blocks_base64: codec::encode_blocks_base64(&data.blocks),
            total: data.blocks.len() as u64,
            non_air,
            counts,
        })
    })
    .await
    .map_err(|e| format!("投影解析任务异常: {}", e))?
}

/// 导出投影到指定格式（重新读取源文件，避免回传整份方块数据）
pub async fn export(params: SchematicExportParams) -> Result<SchematicExportResult, String> {
    let source = PathBuf::from(&params.source_path);
    if !source.is_file() {
        return Err("源投影文件不存在".to_string());
    }
    let target = PathBuf::from(&params.target_path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let format = params.format.clone();
    tokio::task::spawn_blocking(move || {
        let data = read::read_schematic(&source)?;
        let bytes = write::export(&data, &format, &target)?;
        Ok(SchematicExportResult {
            path: target.to_string_lossy().to_string(),
            bytes,
        })
    })
    .await
    .map_err(|e| format!("投影导出任务异常: {}", e))?
}
