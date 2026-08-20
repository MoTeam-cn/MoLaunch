//! 资源包可视化编辑器：打开（zip/folder → 工作目录 + 结构树）与单文件读取。
//!
//! - zip 包：打开时解压到临时工作目录（安全解压，复用 convert::unzip_to_dir），
//!   后续读取全部基于工作目录，保存/导出（M2）才重新打包；
//! - folder 包：直接以原目录作为工作目录；
//! - 路径安全：读取路径先拦截 `..` 跳转，再经 canonicalize 校验必须位于工作目录内。

use std::collections::BTreeMap;

use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

pub(crate) use super::super::types::{
    RpExportParams, RpExportResult, RpOpenParams, RpOpenResult, RpPackFormatInfoParams,
    RpPackFormatInfoResult, RpReadManyParams, RpReadManyResult, RpReadParams, RpReadResult,
    RpTreeNode, RpVersionPackFormatParams, RpVersionPackFormatResult, RpWriteParams, RpWriteResult,
};

mod io;
mod parse;
mod tree;

pub(crate) use parse::{pack_format_info_inner, version_pack_format_inner};
pub(crate) use tree::empty_tree;

// 测试经 super::* 访问子模块实现
#[cfg(test)]
pub(crate) use io::{export_inner, read_many_inner, write_inner};
#[cfg(test)]
pub(crate) use parse::{classify_file, pack_format_to_version};
#[cfg(test)]
pub(crate) use tree::{build_tree, resolve_in_work_dir};

/// 打开资源包：zip 解压到临时工作目录 / folder 直接使用原目录，返回包信息与结构树
///
/// 失败时返回带 `error` 字段的 `RpOpenResult`（而非 Err），前端在页面内展示原因。
pub async fn rp_open(_state: &AppState, params: RpOpenParams) -> Result<serde_json::Value, String> {
    let result = match io::open_inner(&params) {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[RpOpen] failed: path={} err={}", params.path, e);
            RpOpenResult {
                work_dir: String::new(),
                is_zip: false,
                name: String::new(),
                format: String::new(),
                size: 0,
                icon_data_url: None,
                src_path: None,
                pack_format: None,
                mc_version: None,
                description: None,
                tree: empty_tree(),
                error: e,
            }
        }
    };
    if result.error.is_empty() {
        log_info!(
            "[RpOpen] success: name={} format={} size={}",
            result.name,
            result.format,
            result.size
        );
    }
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 读取包内单文件：png/ogg → base64 data URI，json/lang/文本 → 原文
pub async fn rp_read(_state: &AppState, params: RpReadParams) -> Result<serde_json::Value, String> {
    let result = match io::read_inner(&params) {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[RpRead] failed: rel={} err={}", params.rel_path, e);
            RpReadResult {
                kind: String::new(),
                content: String::new(),
                error: e,
            }
        }
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 批量读取模型与关联纹理：blockstate → 模型（含 parent 链）→ 被引用纹理 png，
/// 供前端构建 lodestone Resources 做 3D 预览。缺失/损坏的关联文件静默跳过，不阻塞整批。
pub async fn rp_read_many(
    _state: &AppState,
    params: RpReadManyParams,
) -> Result<serde_json::Value, String> {
    let result = match io::read_many_inner(&params) {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[RpReadMany] failed: root={} err={}", params.rel_path, e);
            RpReadManyResult {
                root: params.rel_path.clone(),
                files: BTreeMap::new(),
                error: e,
            }
        }
    };
    if result.error.is_empty() {
        log_info!(
            "[RpReadMany] success: root={} files={}",
            result.root,
            result.files.len()
        );
    }
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 查询 pack_format 对应的 MC 版本（供前端表单输入时联动校验提示）
pub async fn rp_pack_format_info(
    _state: &AppState,
    params: RpPackFormatInfoParams,
) -> Result<serde_json::Value, String> {
    let result = pack_format_info_inner(params.pack_format);
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 由 MC 版本推导 pack_format（复用皮肤资源包的版本映射，前端下拉选版本后自动回填）
pub async fn rp_version_pack_format(
    _state: &AppState,
    params: RpVersionPackFormatParams,
) -> Result<serde_json::Value, String> {
    let result = version_pack_format_inner(&params.mc_version);
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 写回包内单文件：text 原文 / base64 二进制（复用 rp_read 的路径安全校验）
pub async fn rp_write(
    _state: &AppState,
    params: RpWriteParams,
) -> Result<serde_json::Value, String> {
    let result = match io::write_inner(&params) {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[RpWrite] failed: rel={} err={}", params.rel_path, e);
            RpWriteResult {
                success: false,
                message: e,
            }
        }
    };
    if result.success {
        log_info!(
            "[RpWrite] success: rel={} kind={}",
            params.rel_path,
            params.kind
        );
    }
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 导出资源包：zip 会话保存回原 zip / 另存为 zip（folder 会话保存即直写，仅另存为走此接口）
pub async fn rp_export(
    _state: &AppState,
    params: RpExportParams,
) -> Result<serde_json::Value, String> {
    let result = match io::export_inner(&params).await {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[RpExport] failed: path={} err={}", params.path, e);
            RpExportResult {
                success: false,
                output_path: params.path.clone(),
                message: e,
            }
        }
    };
    if result.success {
        log_info!("[RpExport] success: path={}", result.output_path);
    }
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "explore_test.rs"]
mod explore_test;
