//! Forge loader module

mod install;
mod legacy;
mod list;

pub use install::install;
pub use list::list_versions;