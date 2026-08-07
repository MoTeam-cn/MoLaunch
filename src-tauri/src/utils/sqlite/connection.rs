//! SQLite 全局连接维护。

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use rusqlite::Connection;

use super::migration::{open_and_migrate, TableDef};

struct SqliteDb {
    conn: Mutex<Connection>,
}

static GLOBAL_DB: OnceLock<SqliteDb> = OnceLock::new();

/// 挂载全局数据库连接（幂等）：打开/创建数据库并应用声明式迁移
///
/// 已挂载时直接返回；并发场景下仅首次挂载生效。
pub fn mount(path: &Path, tables: &[TableDef]) -> Result<(), String> {
    if GLOBAL_DB.get().is_some() {
        return Ok(());
    }
    let conn = open_and_migrate(path, tables)?;
    let db = SqliteDb {
        conn: Mutex::new(conn),
    };
    // 并发时可能已被其他线程抢先挂载，丢弃本次结果（数据一致，不冲突）
    let _ = GLOBAL_DB.set(db);
    Ok(())
}

/// 全局连接是否已挂载
pub fn is_mounted() -> bool {
    GLOBAL_DB.get().is_some()
}

/// 在全局连接上执行操作（加锁）
///
/// 闭包内不得再次调用 [`with_conn`]（Mutex 非重入，会死锁）；
/// 需要复用连接内部逻辑时，请拆分为接收 `&Connection` 的内部函数。
pub fn with_conn<T>(f: impl FnOnce(&Connection) -> Result<T, String>) -> Result<T, String> {
    let db = GLOBAL_DB
        .get()
        .ok_or_else(|| "数据库未挂载，请先初始化".to_string())?;
    let conn = db.conn.lock().map_err(|_| "数据库连接被占用".to_string())?;
    f(&conn)
}
