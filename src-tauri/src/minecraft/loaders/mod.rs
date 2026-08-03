//! Loader management module

pub mod fabric_api;
pub mod forge_html;
pub mod forge_installer;

mod api;
mod fabric;
mod forge;
mod liteloader;
mod neoforge;
mod optifine;
mod shared;
mod types;

pub use api::{
    install_loader, list_fabric_versions, list_forge_versions, list_liteloader_versions,
    list_neoforge_versions, list_optifine_versions,
};
pub use types::{LoaderType, LoaderVersion};