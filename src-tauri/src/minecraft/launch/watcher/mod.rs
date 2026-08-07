//! 游戏进程监控器
//!
//! 监控游戏状态与崩溃检测；子模块：process / scheduler / types / log_parser / analyzer / window_title。

mod analyzer;
mod log_parser;
mod log_reader;
mod process;
mod scheduler;
mod types;
mod window_title;

pub use process::GameWatcher;
pub use scheduler::ONLINE_MC_PORT_DETECTED_EVENT;
pub use types::{CrashCategory, CrashInfo, ExitInfo, GameState, LoadProgress, LogEntry, LogLevel};
