//! SQLite schema 定义与迁移。

use std::collections::HashSet;
use std::path::Path;

use rusqlite::Connection;

use crate::log_debug;

/// 列定义（声明式 schema 的一部分）
pub struct ColumnDef {
    /// 列名
    pub name: &'static str,
    /// 列类型与约束（用于 `ALTER TABLE ADD COLUMN`，如 `"INTEGER"`、`"TEXT"`）
    pub column_type: &'static str,
}

/// 表定义（声明式 schema）
pub struct TableDef {
    /// 表名
    pub name: &'static str,
    /// 建表 SQL（应使用 `CREATE TABLE IF NOT EXISTS`，新库首次执行）
    pub create_sql: &'static str,
    /// 可迁移列：挂载时自动检测，缺失则补列，多余则删列
    pub columns: &'static [ColumnDef],
    /// 保留列：由 create_sql 固定定义，迁移时绝不删除
    pub preserved: &'static [&'static str],
}

/// 打开（或创建）数据库并应用声明式迁移：建表 + 自动补全/去除列
pub fn open_and_migrate(path: &Path, tables: &[TableDef]) -> Result<Connection, String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let conn = Connection::open(path).map_err(|e| format!("打开数据库失败: {}", e))?;
    for table in tables {
        migrate_table(&conn, table)?;
    }
    Ok(conn)
}

/// 对单张表应用迁移：建表（IF NOT EXISTS）+ 补全声明列 + 去除多余列
fn migrate_table(conn: &Connection, table: &TableDef) -> Result<(), String> {
    conn.execute_batch(table.create_sql)
        .map_err(|e| format!("初始化数据表失败({}): {}", table.name, e))?;

    let existing = existing_columns(conn, table.name)?;
    for col in table.columns {
        if !existing.contains(&col.name.to_ascii_lowercase()) {
            let sql = format!(
                "ALTER TABLE {} ADD COLUMN {} {};",
                table.name, col.name, col.column_type
            );
            conn.execute_batch(&sql)
                .map_err(|e| format!("迁移数据表失败({}.{}): {}", table.name, col.name, e))?;
            log_debug!("[sqlite] 表 {} 已补列 {}", table.name, col.name);
        }
    }

    let retained: HashSet<String> = table
        .columns
        .iter()
        .map(|c| c.name.to_ascii_lowercase())
        .chain(table.preserved.iter().map(|c| c.to_ascii_lowercase()))
        .collect();
    for name in existing {
        if !retained.contains(&name) {
            match conn.execute_batch(&format!("ALTER TABLE {} DROP COLUMN {};", table.name, name)) {
                Ok(_) => log_debug!("[sqlite] 表 {} 已删列 {}", table.name, name),
                Err(e) => log_debug!(
                    "[sqlite] 表 {} 删列 {} 失败（忽略）: {}",
                    table.name,
                    name,
                    e
                ),
            }
        }
    }
    Ok(())
}

fn existing_columns(conn: &Connection, table: &str) -> Result<HashSet<String>, String> {
    let sql = format!("PRAGMA table_info({})", table);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("检查表结构失败: {}", e))?;
    let cols = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("检查表结构失败: {}", e))?;
    let mut set = HashSet::new();
    for col in cols {
        set.insert(
            col.map_err(|e| format!("读取表结构失败: {}", e))?
                .to_ascii_lowercase(),
        );
    }
    Ok(set)
}
