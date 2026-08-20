//! class 判定 prompt 构建与 AI 响应宽容解析

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::log_warn;
use crate::mod_translation::prompt;
use crate::mod_translation::quality;
use crate::mod_translation::types::{has_chinese, ClassCandidate, JarInspection};

pub(crate) fn build_class_prompt(
    inspection: &JarInspection,
    batch: &[&ClassCandidate],
    last_error: Option<&str>,
) -> String {
    let candidates: Vec<Value> = batch
        .iter()
        .map(|c| serde_json::json!({"id": c.id, "path": c.path, "text": c.text, "paths": c.paths}))
        .collect();
    let mut prompt_value = serde_json::json!({
        "task": "判断 Minecraft 模组 class 常量文本是否展示给玩家；translate 必须提供含简体中文的 translation 且占位符原样保留，exclude 必须提供 reason",
        "output": "只输出 JSON 对象：{\"decisions\":[{\"id\":\"候选id\",\"action\":\"translate\"|\"exclude\",\"translation\":\"译文\",\"reason\":\"理由\"}]}。id 是 24 位十六进制字符串，必须逐字符原样复制 candidates 中的 id，禁止增删改任何字符；每个候选恰好一个 decision，不得遗漏、不得新增。注意：本任务输出 decisions 数组，不是 translations 数组。",
        "loader": inspection.loader.as_str(),
        "modIds": inspection.mod_ids,
        "candidates": candidates,
    });
    if let Some(error) = last_error {
        prompt_value["retryNote"] = format!("上次判定校验失败：{error}，请重发合法 JSON").into();
    }
    prompt_value.to_string()
}

fn str_at<'a>(item: &'a Value, key: &str) -> &'a str {
    item.get(key).and_then(Value::as_str).unwrap_or("")
}

/// 编辑距离 ≤1 判断（id 为 ASCII hex，按字节比较）
pub(crate) fn edit_distance_at_most_1(a: &str, b: &str) -> bool {
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    if long.len() - short.len() > 1 {
        return false;
    }
    if short.len() == long.len() {
        return short
            .bytes()
            .zip(long.bytes())
            .filter(|(x, y)| x != y)
            .count()
            <= 1;
    }
    let (mut i, mut j, mut skipped) = (0usize, 0usize, false);
    while i < short.len() && j < long.len() {
        if short.as_bytes()[i] == long.as_bytes()[j] {
            i += 1;
            j += 1;
        } else if !skipped {
            skipped = true;
            j += 1;
        } else {
            return false;
        }
    }
    true
}

/// 模型可能改写/截断 id：先精确匹配，再按编辑距离 ≤1 唯一匹配兜底
fn resolve_candidate_id<'a>(expected: &'a HashSet<&str>, raw: &str) -> Option<&'a str> {
    if expected.contains(raw) {
        return expected.iter().copied().find(|id| *id == raw);
    }
    let mut matches = expected
        .iter()
        .filter(|id| edit_distance_at_most_1(id, raw));
    let first = matches.next()?;
    if matches.next().is_some() {
        return None; // 不唯一，无法确定
    }
    Some(*first)
}

/// 宽容解析：未知/重复/无效 decision 逐条丢弃并记录 WARN，不整批失败
/// 返回 (有效 decisions, 未覆盖候选)。未覆盖候选由调用方单独请求或跳过。
pub(crate) fn parse_and_validate_decisions<'a>(
    content: &str,
    candidates: &'a [&'a ClassCandidate],
) -> Result<(Vec<DecisionEntry>, Vec<&'a ClassCandidate>), String> {
    let stripped = prompt::strip_json_fences(content);
    let json_str = prompt::extract_json_object(stripped).ok_or("AI 响应中未找到 JSON 对象")?;
    let value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("解析 class 判定 JSON 失败: {e}"))?;
    let items = value
        .get("decisions")
        .and_then(Value::as_array)
        .ok_or("AI 响应缺少 decisions 数组")?;
    let expected: HashSet<&str> = candidates.iter().map(|c| c.id.as_str()).collect();
    let by_id: HashMap<_, _> = candidates.iter().map(|c| (c.id.as_str(), *c)).collect();
    let mut seen: HashSet<&str> = HashSet::new();
    let mut covered: HashSet<&str> = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        let raw_id = str_at(item, "id");
        let Some(id) = resolve_candidate_id(&expected, raw_id) else {
            log_warn!("[ModTranslation] 丢弃未知 class 候选：{raw_id}");
            continue;
        };
        if !seen.insert(id) {
            log_warn!("[ModTranslation] 丢弃重复 class 候选：{id}");
            continue;
        }
        let action = str_at(item, "action");
        let reason = str_at(item, "reason");
        let candidate = by_id
            .get(id)
            .copied()
            .ok_or_else(|| format!("候选缺失：{id}"))?;
        match action {
            "exclude" => {
                if reason.trim().is_empty() {
                    log_warn!("[ModTranslation] 丢弃缺少理由的 class 候选：{id}");
                    continue;
                }
                result.push(DecisionEntry {
                    id: id.to_string(),
                    action: "exclude".to_string(),
                    translation: None,
                    reason: Some(reason.to_string()),
                });
                covered.insert(id);
            }
            "translate" => {
                let raw = str_at(item, "translation");
                let translation = quality::normalize_model_translation(&candidate.text, raw);
                if !has_chinese(&translation) {
                    log_warn!("[ModTranslation] 丢弃译文不含中文的 class 候选：{id}");
                    continue;
                }
                if let Some(error) =
                    quality::validate_protected_tokens(&candidate.text, &translation)
                {
                    log_warn!("[ModTranslation] 丢弃占位符不符的 class 候选 {id}：{error}");
                    continue;
                }
                result.push(DecisionEntry {
                    id: id.to_string(),
                    action: "translate".to_string(),
                    translation: Some(translation),
                    reason: Some(reason.to_string()),
                });
                covered.insert(id);
            }
            other => {
                log_warn!("[ModTranslation] 丢弃未知动作的 class 候选 {id}：{other}");
                continue;
            }
        }
    }
    let uncovered: Vec<&ClassCandidate> = candidates
        .iter()
        .copied()
        .filter(|c| !covered.contains(c.id.as_str()))
        .collect();
    Ok((result, uncovered))
}

/// 单条 class 处置决策（AI 响应解析产物）
pub(crate) struct DecisionEntry {
    pub id: String,
    pub action: String,
    pub translation: Option<String>,
    pub reason: Option<String>,
}
