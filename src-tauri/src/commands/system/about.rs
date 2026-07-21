//! 关于页面数据命令
//!
//! 从 `resources/about/` 下嵌入的 markdown 表格文本加载关于页面所需数据
//! （鸣谢列表、技术栈、许可声明），返回给前端展示。
//!
//! 数据存储为 markdown 表格格式的 .txt 文件，由 `utils::markdown_table` 模块解析，
//! 修改数据只需更新 txt 文件并重新编译后端，无需改动业务代码。

use crate::resources::read_resource;
use crate::utils::markdown_table::parse_markdown_table;
use serde::Serialize;

// ============================================================
// 数据类型
// ============================================================

/// 特别鸣谢项
#[derive(Debug, Serialize, Clone)]
pub struct AcknowledgementItem {
    /// 项目名称
    pub name: String,
    /// 官网地址
    pub home: String,
    /// 简介
    pub desc: String,
    /// logo 资源文件名（位于 src/assets/AboutIcon/）
    pub logo: String,
    /// 作者列表（多个用英文逗号分隔，可能为空）
    pub authors: Vec<String>,
}

/// 技术栈依赖项（前端运行时依赖 / 前端开发工具链 / 后端依赖 共用此结构）
#[derive(Debug, Serialize, Clone)]
pub struct DependencyItem {
    /// 依赖名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 官网/仓库地址
    pub url: String,
    /// 简介
    pub desc: String,
}

/// 许可与版权声明项
#[derive(Debug, Serialize, Clone)]
pub struct LicenseItem {
    /// 依赖名称
    pub name: String,
    /// 版权声明
    pub copyright: String,
    /// 许可类型
    pub license: String,
    /// 来源网站
    #[serde(rename = "sourceUrl")]
    pub source_url: String,
    /// 许可文档地址
    #[serde(rename = "licenseUrl")]
    pub license_url: String,
}

/// 关于页面完整数据
#[derive(Debug, Serialize)]
pub struct AboutData {
    /// 特别鸣谢列表
    pub acknowledgements: Vec<AcknowledgementItem>,
    /// 前端运行时依赖
    #[serde(rename = "frontendDeps")]
    pub frontend_deps: Vec<DependencyItem>,
    /// 前端开发工具链
    #[serde(rename = "frontendDevDeps")]
    pub frontend_dev_deps: Vec<DependencyItem>,
    /// 后端依赖
    #[serde(rename = "backendDeps")]
    pub backend_deps: Vec<DependencyItem>,
    /// 许可与版权声明列表
    pub licenses: Vec<LicenseItem>,
}

// ============================================================
// 解析辅助
// ============================================================

/// 从 HashMap 取字段，缺失返回空字符串
fn get_field(row: &std::collections::HashMap<String, String>, key: &str) -> String {
    row.get(key).cloned().unwrap_or_default()
}

/// 将 authors 字段（英文逗号分隔）解析为 Vec<String>，去除两端空白，跳过空项
fn parse_authors(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// ============================================================
// IPC 命令
// ============================================================

/// 一次性返回关于页面所需的全部数据
///
/// 数据源：`resources/about/` 下的 5 个 txt 文件（include_str! 嵌入二进制）
#[tauri::command]
pub async fn get_about_data() -> Result<AboutData, String> {
    let ack_text = read_resource("about/acknowledgements.txt").map_err(|e| e.to_string())?;
    let fe_text = read_resource("about/frontend-deps.txt").map_err(|e| e.to_string())?;
    let fedev_text = read_resource("about/frontend-dev-deps.txt").map_err(|e| e.to_string())?;
    let be_text = read_resource("about/backend-deps.txt").map_err(|e| e.to_string())?;
    let lic_text = read_resource("about/licenses.txt").map_err(|e| e.to_string())?;

    let acknowledgements = parse_markdown_table(&ack_text)
        .into_iter()
        .map(|row| AcknowledgementItem {
            name: get_field(&row, "name"),
            home: get_field(&row, "home"),
            desc: get_field(&row, "desc"),
            logo: get_field(&row, "logo"),
            authors: parse_authors(&get_field(&row, "authors")),
        })
        .collect();

    let frontend_deps = parse_dependency_table(&fe_text);
    let frontend_dev_deps = parse_dependency_table(&fedev_text);
    let backend_deps = parse_dependency_table(&be_text);

    let licenses = parse_markdown_table(&lic_text)
        .into_iter()
        .map(|row| LicenseItem {
            name: get_field(&row, "name"),
            copyright: get_field(&row, "copyright"),
            license: get_field(&row, "license"),
            source_url: get_field(&row, "sourceUrl"),
            license_url: get_field(&row, "licenseUrl"),
        })
        .collect();

    Ok(AboutData {
        acknowledgements,
        frontend_deps,
        frontend_dev_deps,
        backend_deps,
        licenses,
    })
}

/// 解析技术栈表格（前端运行时 / 前端开发工具链 / 后端依赖 共用结构）
fn parse_dependency_table(text: &str) -> Vec<DependencyItem> {
    parse_markdown_table(text)
        .into_iter()
        .map(|row| DependencyItem {
            name: get_field(&row, "name"),
            version: get_field(&row, "version"),
            url: get_field(&row, "url"),
            desc: get_field(&row, "desc"),
        })
        .collect()
}
