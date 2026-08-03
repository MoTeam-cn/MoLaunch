//! Launch pipeline - 完整的 Minecraft 启动流程
//!
//! 支持并行执行与进度追踪。子模块：types / execute / validate / java_check /
//! natives / pre_launch / process_spawn / pipeline。

mod execute;
mod java_check;
mod natives;
mod pre_launch;
mod process_spawn;
mod runner;
mod types;
mod validate;

pub use self::types::{LaunchConfig, LaunchError, LaunchProgress, LaunchResult, LaunchStage};
pub use runner::LaunchPipeline;