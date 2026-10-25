//! 任务编排：准备（解包 + 分析 + 续传）与执行（多阶段翻译 + 打包）

mod prepare;
mod run;

pub(super) use prepare::{prepare, Prepared};
pub(super) use run::run_task;
