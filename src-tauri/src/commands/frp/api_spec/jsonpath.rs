//! JSONPath 解析与提取
//!
//! 支持的语法（覆盖 endpoints.json 所有用例）：
//! - `$.field` — 从根对象取字段
//! - `$.a.b.c` — 嵌套字段
//! - `$.data[*]` — 数组展平（取出数组所有元素）
//! - `$.data[*].proxies[*]` — 多级数组展平
//! - `$.data.config` — 嵌套字段取值
//!
//! 不支持过滤器、切片等复杂 JSONPath 语法（厂商规范不需要）。

use serde_json::Value;

/// 按 JSONPath 从 JSON Value 取单个值
///
/// 返回 None 表示路径不存在或类型不匹配。
pub fn extract(value: &Value, path: &str) -> Option<Value> {
    let segments = parse_path(path).ok()?;
    traverse(value, &segments)
}

/// 按 JSONPath 从 JSON Value 取数组（展平所有 [*] 段）
///
/// 用于 itemsField 提取（如 `$.data[*].proxies[*]`）。
/// 返回空 Vec 表示路径不存在或无可迭代节点。
pub fn extract_array(value: &Value, path: &str) -> Result<Vec<Value>, String> {
    let segments = parse_path(path)?;
    let mut results = Vec::new();
    traverse_array(value, &segments, &mut results);
    Ok(results)
}

/// 解析 JSONPath 为段列表
///
/// `$.a.b[*].c` → [Field("a"), Field("b"), ArrayAll, Field("c")]
fn parse_path(path: &str) -> Result<Vec<PathSegment>, String> {
    let path = path
        .strip_prefix("$.")
        .or_else(|| path.strip_prefix("$"))
        .ok_or_else(|| format!("JSONPath 必须以 $ 开头: {}", path))?;
    if path.is_empty() {
        return Ok(Vec::new());
    }

    let mut segments = Vec::new();
    // 按 . 分割，但需处理 [*] 后可能紧跟 . 的情况
    for part in path.split('.') {
        if part.is_empty() {
            continue;
        }
        if part == "*" {
            segments.push(PathSegment::ArrayAll);
            continue;
        }
        // 处理 field[*] 或 field[*][*] 形式
        let mut remaining = part;
        while !remaining.is_empty() {
            if let Some(bracket_start) = remaining.find('[') {
                let field = &remaining[..bracket_start];
                if !field.is_empty() {
                    segments.push(PathSegment::Field(field.to_string()));
                }
                let bracket_end = remaining
                    .find(']')
                    .ok_or_else(|| format!("未闭合的 [: {}", remaining))?;
                let inside = &remaining[bracket_start + 1..bracket_end];
                if inside == "*" {
                    segments.push(PathSegment::ArrayAll);
                } else {
                    // 不支持具体索引（如 [0]），厂商规范不使用
                    return Err(format!("不支持的具体索引: [{}]", inside));
                }
                remaining = &remaining[bracket_end + 1..];
            } else {
                segments.push(PathSegment::Field(remaining.to_string()));
                break;
            }
        }
    }
    Ok(segments)
}

/// 路径段
#[derive(Debug, Clone)]
enum PathSegment {
    /// 对象字段
    Field(String),
    /// 数组所有元素（`[*]`）
    ArrayAll,
}

/// 递归遍历取单个值（遇 ArrayAll 取第一个元素）
fn traverse(value: &Value, segments: &[PathSegment]) -> Option<Value> {
    if segments.is_empty() {
        return Some(value.clone());
    }
    match &segments[0] {
        PathSegment::Field(name) => value.get(name).and_then(|v| traverse(v, &segments[1..])),
        PathSegment::ArrayAll => {
            if let Value::Array(arr) = value {
                arr.first().and_then(|v| traverse(v, &segments[1..]))
            } else {
                None
            }
        }
    }
}

/// 递归遍历收集数组（展平所有 ArrayAll 段）
fn traverse_array(value: &Value, segments: &[PathSegment], results: &mut Vec<Value>) {
    if segments.is_empty() {
        results.push(value.clone());
        return;
    }
    match &segments[0] {
        PathSegment::Field(name) => {
            if let Some(v) = value.get(name) {
                traverse_array(v, &segments[1..], results);
            }
        }
        PathSegment::ArrayAll => {
            if let Value::Array(arr) = value {
                for item in arr {
                    traverse_array(item, &segments[1..], results);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_simple() {
        let v = json!({ "access_token": "abc123" });
        assert_eq!(
            extract(&v, "$.access_token"),
            Some(json!("abc123"))
        );
    }

    #[test]
    fn test_extract_nested() {
        let v = json!({ "data": { "config": "frpc content" } });
        assert_eq!(
            extract(&v, "$.data.config"),
            Some(json!("frpc content"))
        );
    }

    #[test]
    fn test_extract_array_flat() {
        let v = json!({ "data": [{ "id": 1 }, { "id": 2 }] });
        let arr = extract_array(&v, "$.data[*]").unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["id"], 1);
    }

    #[test]
    fn test_extract_array_nested() {
        let v = json!({
            "data": [
                { "proxies": [{ "id": "a" }, { "id": "b" }] },
                { "proxies": [{ "id": "c" }] }
            ]
        });
        let arr = extract_array(&v, "$.data[*].proxies[*]").unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["id"], "a");
        assert_eq!(arr[2]["id"], "c");
    }

    #[test]
    fn test_extract_missing() {
        let v = json!({ "a": 1 });
        assert_eq!(extract(&v, "$.b"), None);
    }
}
