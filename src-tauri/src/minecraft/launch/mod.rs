//! Game launch module
//!
//! This module provides Minecraft launch functionality:
//! - Build launch arguments (JVM args, game args, classpath)
//! - Launch game process
//! - Version isolation support
//! - Complete launch pipeline (inspired by PCL2)
//! - Game process monitoring and crash detection
//!
//! Architecture:
//! - pipeline.rs:  完整的启动流水线，支持并行执行和进度追踪
//! - watcher.rs:   游戏进程监控和崩溃检测
//! - mod.rs:       模块入口与公共类型（LaunchArguments / AuthInfo）
//! - arguments.rs: 启动参数构建编排入口（build_launch_arguments）
//! - classpath.rs: Classpath 构建（含继承版本递归）
//! - jvm_args.rs:  JVM 参数构建（含 LUA / JLW 拆分）
//! - game_args.rs: 游戏参数构建
//! - embedded.rs:  嵌入资源释放与库检测

use serde::{Deserialize, Serialize};

pub mod pipeline;
pub mod watcher;

mod arguments;
mod classpath;
mod embedded;
mod game_args;
mod jvm_args;

// Re-export pipeline types
pub use pipeline::{
    LaunchConfig, LaunchPipeline, LaunchProgress, LaunchResult as PipelineLaunchResult, LaunchStage,
};

// Re-export watcher types
pub use watcher::{CrashCategory, CrashInfo, ExitInfo, GameState, GameWatcher, LoadProgress};

// Re-export main entry
pub use arguments::build_launch_arguments;

/// Launch arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchArguments {
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub main_class: String,
    pub classpath: String,
    pub version_id: String,
    pub game_dir: String,
    pub assets_dir: String,
    pub asset_index: String,
    pub auth_info: AuthInfo,
}

/// Auth info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub client_token: String,
    pub login_type: String,
}
