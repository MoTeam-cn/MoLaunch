//! 社区资源命令模块
//!
//! 提供搜索、详情、安装三大类 Tauri 命令

pub mod community_config;
pub mod detail;
pub mod install;
pub mod search;
pub mod secure_config;

pub use detail::{get_mcmod_url, get_project_detail, get_project_versions};
pub use install::modpack::install_modpack;
pub use install::resource::{
    download_resource, download_resource_to_path, format_download_filename,
    get_resource_install_path, install_resource,
};
pub use search::{get_category_tags, search_resources};
