//! 轻量 SQLite 工具封装
//!
//! 职责：
//! 1. **声明式 schema 迁移**：调用方以 [`TableDef`]（表名 + 建表 SQL + 可迁移列 + 保留列）描述表结构，
//!    [`mount`] 建表（`CREATE TABLE IF NOT EXISTS`）后自动对比 `PRAGMA table_info`：
//!    - 声明了但表里缺失的列 → `ALTER TABLE ADD COLUMN`（补全）
//!    - 表里存在但声明未保留的列 → `DROP COLUMN`（去除）
//!    后续调整 schema 只需修改表定义，重新挂载即自动迁移，无需手写迁移分支。
//! 2. **全局连接维护**：进程内仅挂载一次连接（[`mount`]），之后通过 [`with_conn`] 复用，
//!    业务层不再按需打开连接。
//! 3. **通用表访问（[`Table`]）**：SQL 语句生成全部集中在本模块，业务层通过
//!    "表 + 列 + 条件"的语义调用完成 CRUD，不出现任何 SQL 字符串。
//!
//! 业务模块不应直接拼接 SQL；数据访问统一经 [`with_conn`] + [`Table`] 入口执行。

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use rusqlite::{params_from_iter, types::Value, Connection, Row};

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
    /// 保留列：由 create_sql 固定定义（如主键/外键/基础字段），迁移时绝不删除
    pub preserved: &'static [&'static str],
}

// ---------------------------------------------------------------------------
// 全局连接维护
// ---------------------------------------------------------------------------

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
    let conn = db
        .conn
        .lock()
        .map_err(|_| "数据库连接被占用".to_string())?;
    f(&conn)
}

// ---------------------------------------------------------------------------
// schema 迁移
// ---------------------------------------------------------------------------

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

    // 补全：声明了但表里缺失的列（必须成功）
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

    // 去除：表里存在但声明未保留的列（删除通常为清理性操作，失败仅告警不阻塞挂载）
    let retained: HashSet<String> = table
        .columns
        .iter()
        .map(|c| c.name.to_ascii_lowercase())
        .chain(table.preserved.iter().map(|c| c.to_ascii_lowercase()))
        .collect();
    for name in existing {
        if !retained.contains(&name) {
            match conn.execute_batch(&format!("ALTER TABLE {} DROP COLUMN {};", table.name, name))
            {
                Ok(_) => log_debug!("[sqlite] 表 {} 已删列 {}", table.name, name),
                Err(e) => {
                    log_debug!("[sqlite] 表 {} 删列 {} 失败（忽略）: {}", table.name, name, e)
                }
            }
        }
    }
    Ok(())
}

/// 读取表实际列名（转小写的集合）
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
        set.insert(col.map_err(|e| format!("读取表结构失败: {}", e))?.to_ascii_lowercase());
    }
    Ok(set)
}

// ---------------------------------------------------------------------------
// 通用表访问（业务层零 SQL）
// ---------------------------------------------------------------------------

/// 查询条件：`column op value`（默认 `=`），供 [`Table`] 的查询/删除接口使用
pub struct Cond<'a> {
    column: &'a str,
    op: &'static str,
    value: Value,
}

impl<'a> Cond<'a> {
    /// 等值条件：`column = value`
    pub fn eq(column: &'a str, value: Value) -> Self {
        Self { column, op: "=", value }
    }

    /// 大于条件：`column > value`
    pub fn gt(column: &'a str, value: Value) -> Self {
        Self { column, op: ">", value }
    }

    /// 小于条件：`column < value`
    pub fn lt(column: &'a str, value: Value) -> Self {
        Self { column, op: "<", value }
    }
}

/// 通用表访问：SQL 生成集中于此，业务层仅提供表名、列名、条件与参数
#[derive(Clone, Copy)]
pub struct Table {
    name: &'static str,
}

impl Table {
    /// 引用一张表
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    /// 插入一行，返回自增主键 id
    ///
    /// `data`：`(列名, 值)` 列表，SQL 由本方法生成
    pub fn insert(&self, conn: &Connection, data: &[(&str, Value)]) -> Result<i64, String> {
        let cols = data.iter().map(|(c, _)| *c).collect::<Vec<_>>();
        let placeholders = vec!["?"; data.len()].join(", ");
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.name,
            cols.join(", "),
            placeholders
        );
        let values: Vec<Value> = data.iter().map(|(_, v)| v.clone()).collect();
        conn.execute(&sql, params_from_iter(values))
            .map_err(|e| format!("写入失败: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    /// 按主键更新若干列，返回受影响行数
    pub fn update_by_id(
        &self,
        conn: &Connection,
        id: i64,
        data: &[(&str, Value)],
    ) -> Result<usize, String> {
        let sets = data
            .iter()
            .map(|(c, _)| format!("{} = ?", c))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("UPDATE {} SET {} WHERE id = ?", self.name, sets);
        let mut values: Vec<Value> = data.iter().map(|(_, v)| v.clone()).collect();
        values.push(Value::from(id));
        conn.execute(&sql, params_from_iter(values))
            .map_err(|e| format!("更新失败: {}", e))
    }

    /// 按主键删除一行，返回受影响行数
    pub fn delete_by_id(&self, conn: &Connection, id: i64) -> Result<usize, String> {
        let sql = format!("DELETE FROM {} WHERE id = ?", self.name);
        conn.execute(&sql, params_from_iter(vec![Value::from(id)]))
            .map_err(|e| format!("删除失败: {}", e))
    }

    /// 按条件删除多行，返回受影响行数
    pub fn delete_where(&self, conn: &Connection, conds: &[Cond]) -> Result<usize, String> {
        if conds.is_empty() {
            return Err("删除条件不能为空".to_string());
        }
        let sql = self.build_select("DELETE", None, conds, None, None);
        let values: Vec<Value> = conds.iter().map(|c| c.value.clone()).collect();
        conn.execute(&sql, params_from_iter(values))
            .map_err(|e| format!("删除失败: {}", e))
    }

    /// 查询多行，映射为 `T`
    ///
    /// - `columns`：要选择的列
    /// - `conds`：WHERE 条件（可为空）
    /// - `order_by`：`(列名, 是否升序)`（可为空）
    /// - `limit`：行数上限（可为空）
    pub fn query<T>(
        &self,
        conn: &Connection,
        columns: &[&str],
        conds: &[Cond],
        order_by: Option<(&str, bool)>,
        limit: Option<i64>,
        mapper: impl FnMut(&Row<'_>) -> Result<T, rusqlite::Error>,
    ) -> Result<Vec<T>, String> {
        let sql = self.build_select("SELECT", Some(columns), conds, order_by, limit);
        let values: Vec<Value> = conds.iter().map(|c| c.value.clone()).collect();
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("查询失败: {}", e))?;
        let rows = stmt
            .query_map(params_from_iter(values), mapper)
            .map_err(|e| format!("查询失败: {}", e))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| format!("读取失败: {}", e))?);
        }
        Ok(items)
    }

    /// 查询首行（无结果时返回 None）
    pub fn query_first<T>(
        &self,
        conn: &Connection,
        columns: &[&str],
        conds: &[Cond],
        order_by: Option<(&str, bool)>,
        mapper: impl FnMut(&Row<'_>) -> Result<T, rusqlite::Error>,
    ) -> Result<Option<T>, String> {
        let mut items = self.query(conn, columns, conds, order_by, Some(1), mapper)?;
        Ok(items.pop())
    }

    /// 按条件计数
    pub fn count(&self, conn: &Connection, conds: &[Cond]) -> Result<i64, String> {
        let sql = self.build_select("SELECT", Some(&["COUNT(*)"]), conds, None, None);
        let values: Vec<Value> = conds.iter().map(|c| c.value.clone()).collect();
        conn.query_row(&sql, params_from_iter(values), |row| row.get(0))
            .map_err(|e| format!("查询失败: {}", e))
    }

    /// 组装 SQL：`{verb} [{cols}] FROM {table} [WHERE ...] [ORDER BY ...] [LIMIT ...]`
    fn build_select(
        &self,
        verb: &str,
        columns: Option<&[&str]>,
        conds: &[Cond],
        order_by: Option<(&str, bool)>,
        limit: Option<i64>,
    ) -> String {
        let mut sql = match columns {
            Some(cols) => format!("{} {} FROM {}", verb, cols.join(", "), self.name),
            None => format!("{} FROM {}", verb, self.name),
        };
        if !conds.is_empty() {
            let where_clause = conds
                .iter()
                .map(|c| format!("{} {} ?", c.column, c.op))
                .collect::<Vec<_>>()
                .join(" AND ");
            sql.push_str(&format!(" WHERE {}", where_clause));
        }
        if let Some((col, asc)) = order_by {
            sql.push_str(&format!(
                " ORDER BY {} {}",
                col,
                if asc { "ASC" } else { "DESC" }
            ));
        }
        if let Some(lim) = limit {
            sql.push_str(&format!(" LIMIT {}", lim));
        }
        sql
    }
}
