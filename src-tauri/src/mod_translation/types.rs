//! 模组翻译：数据结构定义

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 重试信息（前端展示"第 x/y 次重试"）
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryInfo {
    pub attempt: u32,
    pub total: u32,
}

/// 单阶段进度（前端分进度折叠区展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageProgress {
    pub stage: String,
    /// 阶段权重（0-1，未启用阶段不出现）
    pub weight: f64,
    /// 分进度（0-100）
    pub progress: f64,
}

/// 进度回调（分进度 0-100 + 消息 + 重试信息），供各翻译路由共用
pub type ProgressFn = dyn Fn(f64, &str, Option<RetryInfo>) + Send + Sync;

/// 模组加载器（用于翻译提示词上下文）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Loader {
    Fabric,
    NeoForge,
    Forge,
    #[default]
    Unknown,
}

impl Loader {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fabric => "fabric",
            Self::NeoForge => "neoforge",
            Self::Forge => "forge",
            Self::Unknown => "unknown",
        }
    }
}

/// 语言源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LanguageKind {
    /// 标准语言文件 `assets/<ns>/lang/en_us.json`
    Json,
    /// 标准语言文件 `assets/<ns>/lang/en_us.lang` / `.properties`
    KeyValue,
    /// 路径含 en_us 的嵌套 JSON（JSON Pointer 定位叶子字符串）
    StructuredJson,
    /// 路径含 en_us 的自由文本（.txt / .md，按行对齐）
    FreeText,
}

impl LanguageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::KeyValue => "key-value",
            Self::StructuredJson => "structured-json",
            Self::FreeText => "free-text",
        }
    }
}

/// 一个可翻译的语言源（对应 JAR 内一个 en_us 文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSource {
    pub kind: LanguageKind,
    /// 命名空间（标准 lang 文件为 assets 下的命名空间；其余为路径首段）
    pub namespace: String,
    /// 工作区相对路径（源文件）
    pub source_path: String,
    /// 工作区相对路径（目标 zh_cn 文件）
    pub target_path: String,
    /// 待翻译条目：key -> 源文本
    /// - Json/KeyValue：语言键
    /// - StructuredJson：JSON Pointer（如 `/foo/bar`）
    /// - FreeText：行号字符串
    pub entries: BTreeMap<String, String>,
    /// 已存在的目标翻译（断点续传预留，v1 恒为空）
    pub existing_target: BTreeMap<String, String>,
}

impl LanguageSource {
    /// 需要翻译的条目数（排除已存在且含中文的目标）
    pub fn required_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|(key, source)| {
                let existing = self.existing_target.get(*key).map(String::as_str);
                !existing.is_some_and(|t| !t.trim().is_empty() && has_chinese(t))
                    && !source.trim().is_empty()
            })
            .count()
    }
}

/// 判断文本是否含简体中文字符
pub fn has_chinese(text: &str) -> bool {
    text.chars().any(|c| matches!(c, '\u{4e00}'..='\u{9fff}'))
}

/// 已存在的中文语言文件（预检：模组自带 zh_cn/zh_tw 时提示覆盖风险）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistingChinese {
    /// 工作区相对路径
    pub path: String,
    /// 语言标识（zh_cn / zh_tw）
    pub locale: String,
    /// 条目数
    pub entries: usize,
}

/// JAR 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JarInspection {
    pub input_path: PathBuf,
    pub original_filename: String,
    pub loader: Loader,
    pub mod_ids: Vec<String>,
    pub project_names: Vec<String>,
    pub version: Option<String>,
    pub signed: bool,
    pub language_sources: Vec<LanguageSource>,
    pub language_entries: usize,
    pub class_candidates: Vec<ClassCandidate>,
    pub coverage: Vec<ResourceCoverage>,
    pub quote: Quote,
    pub mod_name: Option<ModNameResult>,
    pub existing_chinese: Vec<ExistingChinese>,
    pub warnings: Vec<String>,
}

/// 分析请求参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeParams {
    pub jar_path: String,
}

/// 分析结果（返回前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeResult {
    pub filename: String,
    pub loader: String,
    pub mod_ids: Vec<String>,
    pub project_names: Vec<String>,
    pub version: Option<String>,
    pub signed: bool,
    pub sources: Vec<SourceSummary>,
    pub total_entries: usize,
    pub class_candidates: Vec<ClassCandidate>,
    pub quote: Quote,
    pub coverage: Vec<ResourceCoverage>,
    pub mod_name: Option<ModNameResult>,
    pub existing_chinese: Vec<ExistingChinese>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSummary {
    pub kind: String,
    pub namespace: String,
    pub source_path: String,
    pub target_path: String,
    pub entries: usize,
}

/// class 常量池候选（运行时可见字符串，跨文件按文本聚合）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassCandidate {
    pub id: String,
    pub path: String,
    pub paths: Vec<String>,
    pub occurrences: usize,
    pub text: String,
}

/// token 报价预估（分析阶段展示，供用户评估成本）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub estimated_input_tokens: u64,
    pub estimated_output_tokens: u64,
    pub estimated_tokens: u64,
    pub estimated_calls: u64,
    pub language_batches: u64,
    pub class_batches: u64,
    pub points: u64,
    pub characters: u64,
    pub entries: u64,
}

/// 资源覆盖诊断（每个工作区文件的处置结论）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCoverage {
    pub path: String,
    pub media_type: String,
    pub disposition: String,
    pub target_path: Option<String>,
    pub text_candidates: u64,
    pub reason: String,
}

/// 模组中文名决策结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModNameResult {
    pub name: String,
    pub source: String,
}

/// 任务完成报告（终态快照携带）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationReport {
    pub task_id: String,
    pub ok: bool,
    pub output_path: String,
    pub mod_name: Option<ModNameResult>,
    pub language_attempted: usize,
    pub language_accepted: usize,
    pub class_resolved: usize,
    pub class_total: usize,
    pub warnings: Vec<String>,
}

/// 启动翻译请求参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartParams {
    pub jar_path: String,
    /// 显式指定模型；为空使用默认模型
    pub model: String,
    /// 每批翻译条目数（20/40/80）
    pub batch_size: u32,
    /// 是否生成模组中文名（默认开）
    #[serde(default = "default_true")]
    pub generate_mod_name: bool,
    /// 是否启用质量回修兜底（默认开）
    #[serde(default = "default_true")]
    pub repair_enabled: bool,
    /// 是否翻译 class 常量池文本（默认开）
    #[serde(default = "default_true")]
    pub class_text_enabled: bool,
}

fn default_true() -> bool {
    true
}

/// 任务状态快照（前端轮询/事件共用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSnapshot {
    pub task_id: String,
    /// running / completed / failed / cancelled
    pub status: String,
    /// analyze / research / language / repair / class / validation / package
    pub stage: String,
    /// 总进度（0-100，按阶段权重加权计算）
    pub progress: f64,
    /// 当前阶段分进度（0-100）
    pub stage_progress: f64,
    /// 重试信息（重试时携带）
    pub retry: Option<RetryInfo>,
    /// 各阶段进度（前端分进度折叠区展示）
    pub stages: Vec<StageProgress>,
    pub message: String,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub mod_name: Option<ModNameResult>,
    pub report: Option<TranslationReport>,
}

impl TaskSnapshot {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            status: "running".to_string(),
            stage: "analyze".to_string(),
            progress: 0.0,
            stage_progress: 0.0,
            retry: None,
            stages: Vec::new(),
            message: String::new(),
            output_path: None,
            error: None,
            mod_name: None,
            report: None,
        }
    }
}
