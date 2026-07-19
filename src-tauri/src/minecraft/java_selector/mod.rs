//! Java 版本选择算法模块
//! 参考 PCL2 的 Java 版本选择逻辑
//!
//! 子模块组织：
//! - `rules`: MC 版本 → Java 版本约束区间规则
//! - `compat`: Java 兼容性校验与文案
//! - `weight`: Java 版本权重（PCL2 权重系统）
//! - `select`: 最佳 Java 选择算法
//! - `installer`: 加载器安装器专用 Java 选择
//! - `tests`: 单元测试

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
    get_mojang_java_requirement, get_recommended_java_version, get_required_java_version,
    get_java_version_range,
};
pub use select::{select_best_java, select_best_java_with_loader};
pub use weight::get_java_version_weight;
