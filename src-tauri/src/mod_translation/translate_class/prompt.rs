//! class 判定 prompt 构建与 AI 响应宽容解析
//!
//! 宽容原则：未知 / 重复 / 无效的 decision 逐条丢弃并记录 WARN，不整批失败；
//! 未被覆盖的候选交给调用方单独再请求一次或跳过。

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::log_warn;
use crate::mod_translation::prompt;
use crate::mod_translation::quality;
use crate::mod_translation::types::{has_chinese, ClassCandidate, JarInspection};

/// 单条 class 处置决策（AI 响应解析产物）
pub(crate) struct DecisionEntry {
    pub id: String,
    pub action: String,
    pub translation: Option<String>,
    pub reason: Option<String>,
}

pub(crate) fn build_class_prompt(
    inspection: &JarInspection,
    batch: &[&ClassCandidate],
    last_error: Option<&str>,
) -> String {
    let candidates: Vec<Value> = batch
        .iter()
        .map(|candidate| {
            serde_json::json!({
                "id": candidate.id,
                "path": candidate.path,
                "text": candidate.text,
                "paths": candidate.paths,
            })
        })
        .collect();

    let mut payload = serde_json::json!({
        "task": "判断 Minecraft 模组 class 常量文本是否展示给玩家；translate 必须提供含简体中文的 translation 且占位符原样保留，exclude 必须提供 reason",
        "output": "只输出 JSON 对象：{\"decisions\":[{\"id\":\"候选id\",\"action\":\"translate\"|\"exclude\",\"translation\":\"译文\",\"reason\":\"理由\"}]}。id 是 24 位十六进制字符串，必须逐字符原样复制 candidates 中的 id，禁止增删改任何字符；每个候选恰好一个 decision，不得遗漏、不得新增。注意：本任务输出 decisions 数组，不是 translations 数组。",
        "loader": inspection.loader.as_str(),
        "modIds": inspection.mod_ids,
        "candidates": candidates,
    });
    if let Some(error) = last_error {
        payload["retryNote"] = format!("上次判定校验失败：{error}，请重发合法 JSON").into();
    }
    payload.to_string()
}

fn str_at<'a>(item: &'a Value, key: &str) -> &'a str {
    item.get(key).and_then(Value::as_str).unwrap_or("")
}

/// 编辑距离 ≤ 1（id 为 ASCII 十六进制，按字节比较）
pub(crate) fn edit_distance_at_most_1(a: &str, b: &str) -> bool {
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    if long.len() - short.len() > 1 {
        return false;
    }
    if short.len() == long.len() {
        return short
            .bytes()
            .zip(long.bytes())
            .filter(|(left, right)| left != right)
            .count()
            <= 1;
    }
    let mut short_index = 0;
    let mut long_index = 0;
    let mut skipped = false;
    while short_index < short.len() && long_index < long.len() {
        if short.as_bytes()[short_index] == long.as_bytes()[long_index] {
            short_index += 1;
            long_index += 1;
        } else if skipped {
            return false;
        } else {
            skipped = true;
            long_index += 1;
        }
    }
    true
}

/// 模型可能改写 / 截断 id：精确匹配优先，其次编辑距离 ≤ 1 的唯一匹配
fn resolve_candidate_id<'a>(known: &'a HashSet<&str>, raw: &str) -> Option<&'a str> {
    if let Some(exact) = known.iter().copied().find(|id| *id == raw) {
        return Some(exact);
    }
    let mut candidates = known
        .iter()
        .copied()
        .filter(|id| edit_distance_at_most_1(id, raw));
    let first = candidates.next()?;
    candidates.next().is_none().then_some(first)
}

/// 单条 decision 是否可采纳（返回采纳结果或丢弃原因）
fn evaluate(item: &Value, candidate: &ClassCandidate) -> Result<DecisionEntry, &'static str> {
    let action = str_at(item, "action");
    let reason = str_at(item, "reason");
    match action {
        "exclude" => {
            if reason.trim().is_empty() {
                return Err("缺少理由");
            }
            Ok(DecisionEntry {
                id: candidate.id.clone(),
                action: "exclude".to_string(),
                translation: None,
                reason: Some(reason.to_string()),
            })
        }
        "translate" => {
            let raw = str_at(item, "translation");
            let translation = quality::normalize_model_translation(&candidate.text, raw);
            if !has_chinese(&translation) {
                return Err("译文不含中文");
            }
            if quality::validate_protected_tokens(&candidate.text, &translation).is_some() {
                return Err("占位符不一致");
            }
            Ok(DecisionEntry {
                id: candidate.id.clone(),
                action: "translate".to_string(),
                translation: Some(translation),
                reason: Some(reason.to_string()),
            })
        }
        _ => Err("未知动作"),
    }
}

/// 宽容解析：返回 (有效 decisions, 未覆盖候选)
pub(crate) fn parse_and_validate_decisions<'a>(
    content: &str,
    candidates: &'a [&'a ClassCandidate],
) -> Result<(Vec<DecisionEntry>, Vec<&'a ClassCandidate>), String> {
    let stripped = prompt::strip_json_fences(content);
    let json_text = prompt::extract_json_object(stripped).ok_or("AI 响应中未找到 JSON 对象")?;
    let value: Value =
        serde_json::from_str(json_text).map_err(|e| format!("解析 class 判定 JSON 失败: {e}"))?;
    let items = value
        .get("decisions")
        .and_then(Value::as_array)
        .ok_or("AI 响应缺少 decisions 数组")?;

    let known: HashSet<&str> = candidates
        .iter()
        .map(|candidate| candidate.id.as_str())
        .collect();
    let by_id: HashMap<&str, &ClassCandidate> = candidates
        .iter()
        .map(|candidate| (candidate.id.as_str(), *candidate))
        .collect();

    let mut seen: HashSet<&str> = HashSet::new();
    let mut decisions: Vec<DecisionEntry> = Vec::new();

    for item in items {
        let raw_id = str_at(item, "id");
        let Some(id) = resolve_candidate_id(&known, raw_id) else {
            log_warn!("[ModTranslation] 丢弃未知 class 候选：{raw_id}");
            continue;
        };
        if !seen.insert(id) {
            log_warn!("[ModTranslation] 丢弃重复 class 候选：{id}");
            continue;
        }
        let Some(candidate) = by_id.get(id).copied() else {
            log_warn!("[ModTranslation] 候选缺失：{id}");
            continue;
        };
        match evaluate(item, candidate) {
            Ok(decision) => decisions.push(decision),
            Err(reason) => log_warn!("[ModTranslation] 丢弃 class 候选 {id}：{reason}"),
        }
    }

    let covered: HashSet<&str> = decisions
        .iter()
        .map(|decision| decision.id.as_str())
        .collect();
    let uncovered: Vec<&ClassCandidate> = candidates
        .iter()
        .copied()
        .filter(|candidate| !covered.contains(candidate.id.as_str()))
        .collect();

    Ok((decisions, uncovered))
}
