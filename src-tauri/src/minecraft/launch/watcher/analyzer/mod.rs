//! 崩溃分析（运行时日志 + 崩溃报告 + hs_err + latest.log）
//!
//! 流程：Collect → Analyze（三级匹配：crit1 精准 → stack 堆栈 → crit3 宽松）→ Output。

mod analyze;
mod collect;
mod crit1;
mod crit3;
mod stack;
mod util;

pub(crate) use analyze::analyze_crash;