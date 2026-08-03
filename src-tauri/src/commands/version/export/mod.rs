//! 版本整合包导出模块
//! 支持整合包名称/版本号、~20 个可勾选导出选项（游戏本体/设置/Mod/资源包/光影包/存档等）、
//! 动态子选项扫描、联网检查（Modrinth hash + CurseForge fingerprint）、配置文件保存/读取、
//! 生成 Modrinth `modrinth.index.json` + overrides 打包 zip。不含「打包启动器本体」和
//! 「启动器个性化内容」（MoLaunch 无此需求）。

pub mod api;
pub mod config;
pub mod network;
pub mod options;
pub mod scan;
pub mod types;
pub mod zip;

pub use api::{EXPORT_PROGRESS_EVENT, export_modpack, get_export_options};