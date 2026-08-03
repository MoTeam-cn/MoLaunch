//! 系统模块统一分发逻辑（system 域 manager 入口）

mod config;
mod developer;
mod dispatcher;
mod game_dir;
mod updater;

pub use dispatcher::dispatch;
