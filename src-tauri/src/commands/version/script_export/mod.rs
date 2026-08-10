//! 启动脚本导出（Windows .bat / macOS、Linux .sh）
//!
//! 逻辑实现位于 `export`（导出编排），脚本内容构建见 `content`，
//! Java 路径解析见 `resolve_java`。本文件仅聚合入口。

mod content;
mod export;
mod resolve_java;

pub use export::export_launch_script;
