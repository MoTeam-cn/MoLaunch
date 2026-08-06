//! 崩溃分析（运行时日志 + 崩溃报告 + hs_err + latest.log）
//!
//! 架构：Collect（收集各源文本）→ Detect（多路检测器并行提取证据）→ Score（置信度聚合）→ Output。
//! 规则以声明式数据表承载（rules.rs），检测器与评分器解耦。

mod analyze;
mod collect;
mod detector;
mod detector_stack;
mod rules;
mod scorer;
mod util;

pub(crate) use analyze::analyze_crash;
