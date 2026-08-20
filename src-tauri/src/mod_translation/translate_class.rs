//! 模组翻译：class 常量池文本翻译路由（确定性排除 → AI 判定 → 改写写回）

mod apply;
mod prompt;
mod route;

pub(crate) use route::run_class_route;

// 测试经 super::* 访问子模块实现
#[cfg(test)]
pub(crate) use prompt::{edit_distance_at_most_1, parse_and_validate_decisions};
#[cfg(test)]
pub(crate) use route::resolve_deterministic_exclusions;

#[cfg(test)]
#[path = "translate_class_test.rs"]
mod tests;
