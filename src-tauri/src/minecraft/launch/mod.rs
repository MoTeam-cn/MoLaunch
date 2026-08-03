//! 游戏启动模块
//!
//! 提供启动参数构建、进程启动/监控、版本隔离；子模块见 pipeline/watcher/arguments 等。

pub mod pipeline;
pub mod watcher;

mod arguments;
pub(crate) mod classpath;
mod embedded;
mod game_args;
mod jvm_args;
pub mod skin_resourcepack;
mod types;

// Re-export pipeline types
pub use pipeline::{
    LaunchConfig, LaunchPipeline, LaunchProgress, LaunchResult as PipelineLaunchResult, LaunchStage,
};

// Re-export watcher types
pub use watcher::{CrashCategory, CrashInfo, ExitInfo, GameState, GameWatcher, LoadProgress};

// Re-export main entry
pub use arguments::build_launch_arguments;

// Re-export launch types
pub use types::{AuthInfo, LaunchArguments};

// Re-export args sanitize helper
pub(crate) use arguments::sanitize_args_for_log;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
