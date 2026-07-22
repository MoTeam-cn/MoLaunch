//! 示例文件导出（manifest + index.html）
//!
//! - `read_layout_sample`：从嵌入资源读取示例布局内容
//! - `export_plugin_sample`：导出插件示例模板到指定路径
//!
//! 嵌入资源路径（在 `resources.rs` 的 `embedded_text` 中登记）：
//! - `samples/plugin/manifest.json` — 示例插件 manifest
//! - `samples/plugin/index.html` — 示例插件入口 HTML
//! - `samples/layout/layout-sample.json` — JSON 布局示例
//! - `samples/layout/layout-sample.xml` — XML 布局示例
//! - `samples/layout/layout-sample.html` — HTML 布局示例

use crate::error_util::log_err;
use crate::{log_info, resources};
use std::path::{Path, PathBuf};
use std::io::Write;

/// 从嵌入资源读取示例布局内容
///
/// 支持 `json` / `xml` / `html` 三种格式，对应 `samples/layout/layout-sample.<ext>`。
#[tauri::command]
pub async fn read_layout_sample(format: String) -> Result<String, String> {
    let resource_path = match format.as_str() {
        "json" => "samples/layout/layout-sample.json",
        "xml" => "samples/layout/layout-sample.xml",
        "html" => "samples/layout/layout-sample.html",
        _ => return Err(format!("Unsupported layout format: {}", format)),
    };

    resources::read_resource(resource_path).map_err(log_err("Failed to read layout sample"))
}

/// 导出插件示例模板到指定路径
///
/// - `as_zip=false`：直接写入文件夹（`manifest.json` + `index.html`）
/// - `as_zip=true`：使用 zip crate 现场打包到 ZIP 文件
///
/// 嵌入资源：`samples/plugin/manifest.json` + `samples/plugin/index.html`
#[tauri::command]
pub async fn export_plugin_sample(dest_path: String, as_zip: bool) -> Result<(), String> {
    let dest = PathBuf::from(&dest_path);

    if as_zip {
        export_zip(&dest).map_err(log_err("Failed to export plugin sample as ZIP"))?;
        log_info!("插件示例已导出为 ZIP: {}", dest.display());
    } else {
        export_folder(&dest).map_err(log_err("Failed to export plugin sample as folder"))?;
        log_info!("插件示例已导出为文件夹: {}", dest.display());
    }

    Ok(())
}

/// 导出为文件夹（写入 manifest.json + index.html）
fn export_folder(dest_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dest_dir)?;

    let manifest = resources::read_resource("samples/plugin/manifest.json")?;
    let index_html = resources::read_resource("samples/plugin/index.html")?;

    std::fs::write(dest_dir.join("manifest.json"), manifest)?;
    std::fs::write(dest_dir.join("index.html"), index_html)?;

    Ok(())
}

/// 导出为 ZIP 文件（使用 zip crate 现场打包）
fn export_zip(dest_zip: &Path) -> anyhow::Result<()> {
    // 确保父目录存在
    if let Some(parent) = dest_zip.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let manifest = resources::read_resource("samples/plugin/manifest.json")?;
    let index_html = resources::read_resource("samples/plugin/index.html")?;

    let file = std::fs::File::create(dest_zip)?;
    let mut zip = zip::ZipWriter::new(file);

    let options = zip::write::SimpleFileOptions::default();

    zip.start_file("manifest.json", options)?;
    zip.write_all(manifest.as_bytes())?;

    zip.start_file("index.html", options)?;
    zip.write_all(index_html.as_bytes())?;

    zip.finish()?;

    Ok(())
}
