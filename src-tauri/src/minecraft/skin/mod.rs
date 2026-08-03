//! 皮肤与披风管理模块
//!
//! 子模块：avatar（皮肤 URL 解析与头像 PNG 下载）、cape（披风）、upload（上传与 profile 刷新）。
mod avatar;
mod cape;
mod upload;

pub use avatar::{download_skin_png, get_skin_url, parse_skin_cape_info, SkinCapeInfo, SkinInfo};
pub use cape::{download_cape_png, equip_cape, get_cape_url, unequip_cape, CapeInfo};
pub use upload::{fetch_profile, upload_skin};