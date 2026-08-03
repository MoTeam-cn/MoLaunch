//! 整合包格式检测
//! 由 `collect.rs` 收集 zip 条目并按层级扫描，`rules.rs` 负责识别各关键文件格式。

mod collect;
mod rules;

pub(crate) use collect::detect_modpack_format;