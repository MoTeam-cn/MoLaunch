//! 字段映射反序列化器（字符串=字段名，对象={field, split}，模板={account.token}）

/// 字段映射（字符串=字段名，模板字符串={account.token} 引用，对象={field, split} 拆分）
#[derive(Debug, Clone)]
pub struct FieldMapping {
    /// 厂商字段名
    pub field: Option<String>,
    /// 从合并字段拆分的分隔符（如 ":" 从 "host:port" 拆分）
    pub split: Option<String>,
    /// 直接字符串值（如 "{account.token}" 取账号信息 token）
    pub value: Option<String>,
}

impl<'de> serde::Deserialize<'de> for FieldMapping {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Str(String),
            Obj {
                field: Option<String>,
                split: Option<String>,
                value: Option<String>,
            },
        }

        match Repr::deserialize(deserializer)? {
            Repr::Str(s) => {
                // 模板字符串（以 { 开头，如 {account.token}）→ value；否则 → field
                if s.starts_with('{') {
                    Ok(FieldMapping {
                        field: None,
                        split: None,
                        value: Some(s),
                    })
                } else {
                    Ok(FieldMapping {
                        field: Some(s),
                        split: None,
                        value: None,
                    })
                }
            }
            Repr::Obj {
                field,
                split,
                value,
            } => Ok(FieldMapping {
                field,
                split,
                value,
            }),
        }
    }
}