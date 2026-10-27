//! 投影工具：会话式预览、编辑、变换与格式导出
//!
//! 支持 Sponge .schematic（v1/v2）、Litematica .litematic、原版结构 .nbt，
//! 以及 1.13 之前的旧版 MCEdit 投影（数字 id + 元数据）。
//! 三种现代格式与旧版格式均按公开规范自行实现解析与写出，不依赖第三方投影库。
//!
//! 打开后数据常驻内存（会话），前端按视口分片取用，因此不再受整份数据体积限制。

mod codec;
mod entities;
mod legacy_blocks;
mod overview;
mod read;
mod session;
mod volume;
mod write;

#[cfg(test)]
mod schematic_test;

use std::path::PathBuf;

use crate::commands::tools::types::{
    SchematicBlockCount, SchematicCloseParams, SchematicEditParams, SchematicEditResult,
    SchematicExportParams, SchematicExportResult, SchematicHistoryParams, SchematicHistoryResult,
    SchematicOpenParams, SchematicOpenResult, SchematicOverviewParams, SchematicOverviewResult,
    SchematicSliceParams, SchematicSliceResult, SchematicTransformParams, SchematicTransformResult,
    SchematicVolumeParams, SchematicVolumeResult,
};

/// 单次返回的方块统计条目上限
const TOP_COUNT_LIMIT: usize = 30;

/// 打开投影文件，返回会话 ID（方块数据留在内存中按需取用）
pub async fn open(params: SchematicOpenParams) -> Result<SchematicOpenResult, String> {
    let path = PathBuf::from(&params.path);
    if !path.is_file() {
        return Err("投影文件不存在".to_string());
    }
    let data = tokio::task::spawn_blocking(move || read::read_schematic(&path))
        .await
        .map_err(|e| format!("投影解析任务异常: {}", e))??;

    let counts = codec::top_counts(&data.blocks, &data.palette, TOP_COUNT_LIMIT)
        .into_iter()
        .map(|(id, count)| SchematicBlockCount { id, count })
        .collect();
    let non_air = read::non_air_count(&data);
    let result = SchematicOpenResult {
        session_id: String::new(),
        format: data.format.to_string(),
        name: data.name.clone(),
        size: data.size,
        palette: data.palette.clone(),
        total: data.blocks.len() as u64,
        non_air,
        counts,
    };
    let session_id = session::create(data);
    Ok(SchematicOpenResult {
        session_id,
        ..result
    })
}

/// 概览：层密度与俯视高度图（供立体预览与非空层定位）
pub async fn overview(params: SchematicOverviewParams) -> Result<SchematicOverviewResult, String> {
    let data = overview::build(&params.session_id)?;
    Ok(SchematicOverviewResult {
        size: data.size,
        layer_counts: data.layer_counts,
        width: data.width,
        height: data.height,
        step: data.step,
        top_index_base64: overview::encode_u16(&data.top_index),
        top_height_base64: overview::encode_u16(&data.top_height),
        empty_index: overview::EMPTY_INDEX,
    })
}

/// 取整体体素数据（3D 预览用，zlib 压缩后一次传完）
pub async fn volume(params: SchematicVolumeParams) -> Result<SchematicVolumeResult, String> {
    let session_id = params.session_id;
    let built = tokio::task::spawn_blocking(move || volume::build(&session_id))
        .await
        .map_err(|e| format!("体素打包任务异常: {}", e))??;
    Ok(SchematicVolumeResult {
        size: built.size,
        palette: built.palette,
        blocks_base64: built.blocks_base64,
        total: built.total,
        non_air: built.non_air,
        air_index: built.air_index,
    })
}

/// 取指定轴、指定层、指定视口窗口的方块索引
pub async fn slice(params: SchematicSliceParams) -> Result<SchematicSliceResult, String> {
    let (width, height, cells_base64) = session::slice(
        &params.session_id,
        &params.axis,
        params.layer,
        params.x,
        params.y,
        params.w,
        params.h,
    )?;
    Ok(SchematicSliceResult {
        x: params.x,
        y: params.y,
        width,
        height,
        cells_base64,
    })
}

/// 在矩形区域内填充方块
pub async fn edit(params: SchematicEditParams) -> Result<SchematicEditResult, String> {
    let (changed, non_air, can_undo, can_redo) = session::edit(
        &params.session_id,
        session::EditRequest {
            axis: &params.axis,
            layer: params.layer,
            x: params.x,
            y: params.y,
            w: params.w,
            h: params.h,
            block: params.block,
            mode: &params.mode,
            from: params.from,
            shape: &params.shape,
            radius: params.radius,
            height: params.height,
            points: &params.points,
        },
    )?;
    Ok(SchematicEditResult {
        changed,
        non_air,
        can_undo,
        can_redo,
    })
}

/// 撤销 / 重做
pub async fn history(params: SchematicHistoryParams) -> Result<SchematicHistoryResult, String> {
    let (changed, non_air, can_undo, can_redo) =
        session::history(&params.session_id, &params.action)?;
    Ok(SchematicHistoryResult {
        changed,
        non_air,
        can_undo,
        can_redo,
    })
}

/// 旋转 / 镜像 / 上下翻转
pub async fn transform(
    params: SchematicTransformParams,
) -> Result<SchematicTransformResult, String> {
    let (size, non_air) = session::transform(&params.session_id, &params.kind)?;
    Ok(SchematicTransformResult { size, non_air })
}

/// 导出当前会话到指定格式
pub async fn export(params: SchematicExportParams) -> Result<SchematicExportResult, String> {
    let target = PathBuf::from(&params.target_path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let format = params.format.clone();
    let session_id = params.session_id.clone();
    tokio::task::spawn_blocking(move || {
        session::inspect(&session_id, |session| {
            let bytes = write::export(&session.data, &format, &target)?;
            Ok(SchematicExportResult {
                path: target.to_string_lossy().to_string(),
                bytes,
            })
        })
    })
    .await
    .map_err(|e| format!("投影导出任务异常: {}", e))?
}

/// 关闭会话，释放内存
pub async fn close(params: SchematicCloseParams) -> Result<(), String> {
    session::close(&params.session_id);
    Ok(())
}
