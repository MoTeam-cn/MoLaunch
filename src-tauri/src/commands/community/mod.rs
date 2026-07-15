//! 社区资源命令模块
//!
//! 参考 PCL2 PageDownload/Resource 侧栏功能
//! 提供搜索、详情、安装三大类 Tauri 命令

pub mod detail;
pub mod install;
pub mod search;
pub mod secure_config;

pub use detail::{get_project_detail, get_project_versions};
pub use install::{
    download_resource, download_resource_to_path, get_resource_install_path, install_resource,
};
pub use search::{get_category_tags, search_resources};
pub use secure_config::{get_curseforge_config, set_curseforge_config};
