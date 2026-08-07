//! SQLite 通用表访问与条件 SQL 生成。

use rusqlite::{params_from_iter, types::Value, Connection, Row};

/// 查询条件：`column op value`（默认 `=`），供 [`Table`] 的查询/删除接口使用
pub struct Cond<'a> {
    pub(crate) column: &'a str,
    pub(crate) op: &'static str,
    pub(crate) value: Value,
}

impl<'a> Cond<'a> {
    /// 等值条件：`column = value`
    pub fn eq(column: &'a str, value: Value) -> Self {
        Self {
            column,
            op: "=",
            value,
        }
    }

    /// 大于条件：`column > value`
    pub fn gt(column: &'a str, value: Value) -> Self {
        Self {
            column,
            op: ">",
            value,
        }
    }

    /// 小于条件：`column < value`
    pub fn lt(column: &'a str, value: Value) -> Self {
        Self {
            column,
            op: "<",
            value,
        }
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
    pub fn query<T>(
        &self,
        conn: &Connection,
        columns: &[&str],
        conds: &[Cond],
        order_by: Option<(&str, bool)>,
        limit: Option<i64>,
        mut mapper: impl FnMut(&Row<'_>) -> Result<T, rusqlite::Error>,
    ) -> Result<Vec<T>, String> {
        let sql = self.build_select("SELECT", Some(columns), conds, order_by, limit);
        let values: Vec<Value> = conds.iter().map(|c| c.value.clone()).collect();
        let mut stmt = conn.prepare(&sql).map_err(|e| format!("查询失败: {}", e))?;
        let rows = stmt
            .query_map(params_from_iter(values), &mut mapper)
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
