//! NBT 公共命令（parse / save / list_save_files）与普通 NBT 文件保存。

use std::path::Path;

use fastnbt::{SerOpts, Value as NbtValue};

use crate::error_util::log_err;
use crate::log_info;
use crate::state::AppState;

use super::super::types::{
    NbtChunkInfo, NbtListSaveFilesParams, NbtListSaveFilesResult, NbtNode, NbtParseParams,
    NbtParseResult, NbtSaveFileItem, NbtSaveParams, NbtSaveResult,
};
use super::compress::{gunzip_if_needed, gzip_compress};
use super::convert::{convert_nbt, node_to_value};
use super::mca::{parse_mca, save_mca_chunk};
use super::scan::collect_save_files;

/// 解析 NBT / mca 文件，返回 NbtNode 树（或 mca 的区块列表）
///
/// 读取 `params.file_path` 指定的文件：普通 NBT（gzip 或原始）解析为树；
/// .mca 按 Anvil 容器解析，返回全部有效区块的 NBT 树。
pub async fn parse(state: &AppState, params: NbtParseParams) -> Result<serde_json::Value, String> {
    let _ = state; // 当前未使用 state，保留以符合统一命令签名
    let file_path = params.file_path.clone();
    log_info!("[NBT] 解析文件: {}", file_path);

    let (root_name, root_value, file_type, chunks) = tokio::task::spawn_blocking(
        move || -> Result<(String, Option<NbtValue>, String, Vec<NbtChunkInfo>), String> {
            let raw = std::fs::read(&file_path).map_err(log_err("NBT 读取文件失败"))?;
            if raw.is_empty() {
                return Err("NBT 文件为空".to_string());
            }
            // mca 容器：解析全部区块
            if file_path.to_lowercase().ends_with(".mca") {
                let chunks = parse_mca(&raw)?;
                return Ok((String::new(), None, "mca".to_string(), chunks));
            }
            // 普通 NBT：gzip 解压 + fastnbt 解析
            let data = gunzip_if_needed(&raw)?;
            if data.first() == Some(&0u8) {
                return Err("NBT 文件无有效数据（根为 TAG_End）".to_string());
            }
            let root_name = read_root_name(&data);
            let value: NbtValue =
                fastnbt::from_bytes(&data).map_err(|e| format!("fastnbt 解析失败: {}", e))?;
            Ok((root_name, Some(value), "nbt".to_string(), Vec::new()))
        },
    )
    .await
    .map_err(log_err("NBT 解析任务失败"))??;

    let root = match root_value {
        Some(v) => convert_nbt(&root_name, &v),
        None => NbtNode {
            name: String::new(),
            tag_type: "compound".to_string(),
            value: None,
            children: Vec::new(),
        },
    };

    log_info!(
        "[NBT] 解析完成: 类型 {}, {} 个节点 / {} 个区块",
        file_type,
        root.children.len(),
        chunks.len()
    );

    let result = NbtParseResult {
        root,
        file_type,
        chunks,
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 保存 NBT / mca 文件
///
/// 普通 NBT 文件：NbtNode 树序列化后写回（原 gzip 则保持 gzip）。
/// mca 文件：整体重打包（保留其他区块原字节与时间戳表），写回指定区块。
pub async fn save(state: &AppState, params: NbtSaveParams) -> Result<serde_json::Value, String> {
    let _ = state;
    let file_path = params.file_path;
    let fp = file_path.clone();
    let root = params.root;
    let chunk_index = params.chunk_index;
    log_info!("[NBT] 保存文件: {} (chunk: {:?})", file_path, chunk_index);

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        if fp.to_lowercase().ends_with(".mca") {
            let idx = chunk_index.ok_or("mca 文件保存必须指定区块索引")?;
            save_mca_chunk(&fp, idx, &root)
        } else {
            save_nbt_file(&fp, &root)
        }
    })
    .await
    .map_err(log_err("NBT 保存任务失败"))??;

    log_info!("[NBT] 保存成功: {}", file_path);
    let result = NbtSaveResult { success: true };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 列出存档目录内的 NBT 文件（level.dat / playerdata / region .mca 等）
pub async fn list_save_files(
    state: &AppState,
    params: NbtListSaveFilesParams,
) -> Result<serde_json::Value, String> {
    if params.world_name.is_empty()
        || !crate::utils::path::is_safe_relative_path(&params.world_name)
        || params.world_name.contains('/')
        || params.world_name.contains('\\')
    {
        return Err("非法存档名称".to_string());
    }

    let saves_dir =
        super::super::archive::resolve_saves_dir(state, params.version_id.as_deref()).await;
    let world_dir = saves_dir.join(&params.world_name);
    if !world_dir.is_dir() {
        return Err(format!("存档目录不存在: {}", world_dir.display()));
    }

    log_info!("[NBT] 扫描存档目录: {}", world_dir.display());

    let world_dir_clone = world_dir.clone();
    let items = tokio::task::spawn_blocking(move || -> Vec<NbtSaveFileItem> {
        let mut items = Vec::new();
        collect_save_files(&world_dir_clone, "", &mut items);
        items.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        items
    })
    .await
    .map_err(log_err("NBT 存档文件列表任务失败"))?;

    log_info!(
        "[NBT] 存档 {} 内 NBT 文件 {} 个",
        params.world_name,
        items.len()
    );
    let result = NbtListSaveFilesResult { items };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 保存普通 NBT 文件（保持原 gzip 状态，原子写）
fn save_nbt_file(file_path: &str, root: &NbtNode) -> Result<(), String> {
    if root.tag_type != "compound" {
        return Err("NBT 根节点必须为 compound".to_string());
    }
    let raw = std::fs::read(file_path).map_err(log_err("NBT 读取文件失败"))?;
    let gzipped = raw.len() >= 2 && raw[0] == 0x1f && raw[1] == 0x8b;

    let value = node_to_value(root)?;
    let bytes = fastnbt::to_bytes_with_opts(&value, SerOpts::new().root_name(&root.name))
        .map_err(|e| format!("NBT 序列化失败: {}", e))?;
    let out: Vec<u8> = if gzipped {
        gzip_compress(&bytes)?
    } else {
        bytes
    };
    atomic_write(Path::new(file_path), &out)
}

/// 从 NBT 字节流读取根 compound 名称
///
/// NBT 根格式：`[u8 tag_type][u16 name_len][name bytes][payload...]`
/// 长度不足或解析失败时返回空字符串（不阻塞解析）。
fn read_root_name(data: &[u8]) -> String {
    if data.len() < 3 {
        return String::new();
    }
    let len = u16::from_be_bytes([data[1], data[2]]) as usize;
    if data.len() < 3 + len {
        return String::new();
    }
    String::from_utf8_lossy(&data[3..3 + len]).into_owned()
}

/// 原子写文件：先写同目录临时文件再替换目标（避免写一半损坏原文件）
fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("bin");
    let tmp = path.with_extension(format!("{}.tmp", ext));
    std::fs::write(&tmp, data).map_err(log_err("NBT 写入临时文件失败"))?;
    if path.exists() {
        std::fs::remove_file(path).map_err(log_err("NBT 删除原文件失败"))?;
    }
    std::fs::rename(&tmp, path).map_err(log_err("NBT 替换文件失败"))?;
    Ok(())
}

#[cfg(test)]
#[path = "nbt_test.rs"]
mod nbt_test;
