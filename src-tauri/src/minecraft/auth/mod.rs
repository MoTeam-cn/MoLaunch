//! 认证模块

pub mod authlib;
pub mod microsoft;
pub mod storage;

mod offline;
mod types;

pub use offline::{
    adjust_uuid_for_skin_variant, generate_offline_uuid, login_offline, validate_username,
};
pub use types::{LoginResult, LoginType};
