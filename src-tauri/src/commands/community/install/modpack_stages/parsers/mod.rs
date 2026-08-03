//! 各格式整合包 manifest 解析（CF / MR / HMCL / MMC / MCBBS / LauncherPack / Compress）
//! 按格式拆分子模块：curseforge / modrinth / hmcl；其余格式（mmc/mcbbs/
//! launcher_pack/compress）解析函数在 api.rs。

mod api;
mod curseforge;
mod hmcl;
mod modrinth;

pub(super) use api::{parse_compress, parse_launcher_pack, parse_mcbbs, parse_mmc};
pub(super) use curseforge::parse_cf;
pub(super) use hmcl::parse_hmcl;
pub(super) use modrinth::parse_mr;
