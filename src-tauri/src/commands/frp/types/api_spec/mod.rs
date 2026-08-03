//! Open API 接口规范类型（api/endpoints.json 反序列化结构）
//!
//! 拆分为子模块：`models`（顶层 DTO 与认证/端点类型）、`field_mapping`（自定义反序列化）。

mod field_mapping;
mod models;

pub use field_mapping::*;
pub use models::*;
