//! SQLite 工具封装：提供全局连接、声明式 schema 迁移与通用表访问。

mod connection;
mod migration;
mod table;

pub use connection::{is_mounted, mount, with_conn};
pub use migration::{open_and_migrate, ColumnDef, TableDef};
pub use table::{Cond, Table};
