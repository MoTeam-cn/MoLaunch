//! 会话压缩管线（L1 微压缩 + L3 AI 摘要 + 重塑器）
//!
//! 子模块：`trigger`（触发判定）/ `l1`（工具输出截断）/ `l3`（AI 摘要）/
//! `rebuild`（重塑器）/ `pipeline`（总控编排）。
//!
//! 按方案 A 实现：零新依赖，同步触发，摘要落独立表持久化。

mod l1;
mod l3;
mod pipeline;
mod rebuild;
mod trigger;

#[cfg(test)]
mod l1_test;

#[cfg(test)]
mod rebuild_test;

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod trigger_test;

pub use pipeline::compact_if_needed;
pub(super) use trigger::clear_cooldown;
