//! 关于页面数据命令
//! 从 `resources/about/` 下嵌入的 markdown 表格文本加载关于页面所需数据
//! （鸣谢列表、技术栈、许可声明），返回给前端展示。
//! 数据存储为 markdown 表格格式的 .txt 文件，由 `utils::markdown_table` 模块解析，
//! 修改数据只需更新 txt 文件并重新编译后端，无需改动业务代码。

use crate::error_util::log_err;
use crate::resources::read_resource;
use crate::utils::markdown_table::parse_markdown_table;
use serde::Serialize;


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
    /// 作者列表（可能为空数组）
    pub authors: Vec<Author>,
}

/// 作者信息
#[derive(Debug, Serialize, Clone)]
pub struct Author {
    /// 作者姓名
    pub name: String,
    /// 作者头像文件名（位于 src/assets/AboutIcon/），None 表示无头像
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
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


/// 从 HashMap 取字段，缺失返回空字符串
fn get_field(row: &std::collections::HashMap<String, String>, key: &str) -> String {
    row.get(key).cloned().unwrap_or_default()
}

/// 解析 authors 字段为结构化作者列表
///
/// 格式：`姓名1:头像1.png,姓名2:头像2.png,姓名3`
/// - 多个作者用英文逗号分隔
/// - 每位作者用冒号分隔姓名与头像文件名，冒号可省略（表示无头像）
/// - 空字符串返回空 Vec
fn parse_authors(raw: &str) -> Vec<Author> {
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            if let Some((name, avatar)) = s.split_once(':') {
                let name = name.trim().to_string();
                let avatar = avatar.trim().to_string();
                Author {
                    name,
                    avatar: if avatar.is_empty() { None } else { Some(avatar) },
                }
            } else {
                Author {
                    name: s.to_string(),
                    avatar: None,
                }
            }
        })
        .collect()
}


/// 一次性返回关于页面所需的全部数据
///
/// 数据源：`resources/about/` 下的 5 个 txt 文件（include_str! 嵌入二进制）
pub async fn get_about_data() -> Result<AboutData, String> {
    let ack_text = read_resource("about/acknowledgements.txt").map_err(log_err("Failed to read acknowledgements"))?;
    let fe_text = read_resource("about/frontend-deps.txt").map_err(log_err("Failed to read frontend dependencies"))?;
    let fedev_text = read_resource("about/frontend-dev-deps.txt").map_err(log_err("Failed to read frontend dev dependencies"))?;
    let be_text = read_resource("about/backend-deps.txt").map_err(log_err("Failed to read backend dependencies"))?;
    let lic_text = read_resource("about/licenses.txt").map_err(log_err("Failed to read licenses"))?;

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