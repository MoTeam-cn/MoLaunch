//! 配置更新核心逻辑
//! 编排实现位于 `flow`（apply_config_inner + apply_java），域子函数位于 `fields`。

mod fields;
mod flow;

pub(crate) use flow::apply_config_inner;
