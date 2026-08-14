//! 合成配方生成器导出（数据包 zip 打包）
//! 前端生成数据包文件清单（pack.mcmeta + 配方 + 自定义标签），本模块将其打包为 zip 写入用户选择路径。
//! 复用 version/export 的 zip writer 辅助（create_zip_writer / write_string_entry），避免重复实现。

use crate::commands::tools::types::RecipeGeneratorExportParams;
use crate::commands::version::export::zip::helpers::{create_zip_writer, write_string_entry};

/// 把前端生成的数据包文件清单打包为 zip 写入指定路径，返回写入路径
pub fn export_datapack(
    params: RecipeGeneratorExportParams,
) -> Result<serde_json::Value, String> {
    let out_path = std::path::Path::new(&params.path);
    let (mut zip, options) = create_zip_writer(out_path)?;
    for file in &params.files {
        if file.path.is_empty() {
            continue;
        }
        write_string_entry(&mut zip, options, &file.path, &file.content)?;
    }
    zip.finish().map_err(|e| format!("完成 zip 写入失败: {}", e))?;
    serde_json::to_value(params.path).map_err(|e| e.to_string())
}
