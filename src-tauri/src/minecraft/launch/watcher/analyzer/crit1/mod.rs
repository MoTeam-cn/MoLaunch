//! 第一级高优先级精准匹配
//!
//! 按 log_crash → log_mc → log_hs 三个来源依次检查，命中即返回；子模块：collect / rules。

mod collect;
mod rules;

pub(super) use collect::analyze_crit1;
