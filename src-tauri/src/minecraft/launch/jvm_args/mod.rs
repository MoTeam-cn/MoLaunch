//! JVM 参数构建
//!
//! `build_jvm_args` 编排参数拼装（build）与版本规则（rules）。

mod build;
mod rules;

pub(super) use build::build_jvm_args;
