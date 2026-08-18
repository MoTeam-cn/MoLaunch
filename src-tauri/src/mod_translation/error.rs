//! 模组翻译：统一错误类型（稳定错误码 + 可读消息）

use std::fmt;

use serde::{Deserialize, Serialize};

/// 抛给前端的稳定错误码（kebab-case 序列化）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TranslateErrorCode {
    UnsafeArchivePath,
    SignedModRefused,
    InvalidArchive,
    MissingApiKey,
    ModelNotFound,
    EmptyModelResponse,
    InvalidModelResponse,
    PlaceholderMismatch,
    WritebackVerificationFailed,
    QualityHardErrors,
    WorkGraphNoExit,
    UnsupportedResource,
    Cancelled,
    Io,
    Config,
    AiDisabled,
    AiModelNotSelected,
    AiRequestFailed,
}

impl TranslateErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsafeArchivePath => "unsafe-archive-path",
            Self::SignedModRefused => "signed-mod-refused",
            Self::InvalidArchive => "invalid-archive",
            Self::MissingApiKey => "missing-api-key",
            Self::ModelNotFound => "model-not-found",
            Self::EmptyModelResponse => "empty-model-response",
            Self::InvalidModelResponse => "invalid-model-response",
            Self::PlaceholderMismatch => "placeholder-mismatch",
            Self::WritebackVerificationFailed => "writeback-verification-failed",
            Self::QualityHardErrors => "quality-hard-errors",
            Self::WorkGraphNoExit => "work-graph-no-exit",
            Self::UnsupportedResource => "unsupported-resource",
            Self::Cancelled => "cancelled",
            Self::Io => "io",
            Self::Config => "config",
            Self::AiDisabled => "ai-disabled",
            Self::AiModelNotSelected => "ai-model-not-selected",
            Self::AiRequestFailed => "ai-request-failed",
        }
    }
}

/// 翻译错误：稳定错误码 + 可读消息
#[derive(Debug, Clone)]
pub struct TranslateError {
    pub code: TranslateErrorCode,
    pub message: String,
}

impl TranslateError {
    pub fn new(code: TranslateErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for TranslateError {
    fn from(error: std::io::Error) -> Self {
        Self::new(TranslateErrorCode::Io, error.to_string())
    }
}

impl fmt::Display for TranslateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for TranslateError {}

pub type Result<T> = std::result::Result<T, TranslateError>;
