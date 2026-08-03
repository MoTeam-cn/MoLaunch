//! 配置持久化模块
//!
//! 使用 storage 模块管理配置文件（INI 格式）

mod load;
mod save;

pub use load::load_config;
pub use save::save_config;
