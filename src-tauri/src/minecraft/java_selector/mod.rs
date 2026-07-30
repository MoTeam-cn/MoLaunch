//! Java 版本选择算法模块
//!
//! 子模块：rules/compat/weight/select/installer/tests。

mod compat;
mod installer;
mod rules;
mod select;
mod weight;

#[cfg(test)]
mod tests;

// 重新导出公共 API，保持 `crate::minecraft::java_selector::*` 路径稳定
pub use compat::{check_java_compatible, describe_java_requirement};
pub use installer::get_java_for_installer;
pub use rules::{
    get_java_version_range, get_mojang_java_requirement, get_recommended_java_version,
    get_required_java_version,
};
pub use select::{select_best_java, select_best_java_with_loader};
pub use weight::get_java_version_weight;
