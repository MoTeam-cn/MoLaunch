//! 模组翻译：统一错误类型（稳定错误码 + 可读消息）
//!
//! 错误码以字符串形式对前端暴露，改动等同于破坏兼容，只能新增。

use std::fmt;

use serde::Serialize;

/// 稳定错误码（序列化后即该字符串）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct TranslateErrorCode(&'static str);

impl TranslateErrorCode {
    pub const UNSAFE_ARCHIVE_PATH: Self = Self("unsafe-archive-path");
    pub const SIGNED_MOD_REFUSED: Self = Self("signed-mod-refused");
    pub const INVALID_ARCHIVE: Self = Self("invalid-archive");
    pub const MISSING_API_KEY: Self = Self("missing-api-key");
    pub const MODEL_NOT_FOUND: Self = Self("model-not-found");
    pub const EMPTY_MODEL_RESPONSE: Self = Self("empty-model-response");
    pub const INVALID_MODEL_RESPONSE: Self = Self("invalid-model-response");
    pub const PLACEHOLDER_MISMATCH: Self = Self("placeholder-mismatch");
    pub const WRITEBACK_VERIFICATION_FAILED: Self = Self("writeback-verification-failed");
    pub const QUALITY_HARD_ERRORS: Self = Self("quality-hard-errors");
    pub const WORK_GRAPH_NO_EXIT: Self = Self("work-graph-no-exit");
    pub const UNSUPPORTED_RESOURCE: Self = Self("unsupported-resource");
    pub const CANCELLED: Self = Self("cancelled");
    pub const IO: Self = Self("io");
    pub const CONFIG: Self = Self("config");
    pub const AI_DISABLED: Self = Self("ai-disabled");
    pub const AI_MODEL_NOT_SELECTED: Self = Self("ai-model-not-selected");
    pub const AI_REQUEST_FAILED: Self = Self("ai-request-failed");

    pub fn as_str(self) -> &'static str {
        self.0
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
        Self::new(TranslateErrorCode::IO, error.to_string())
    }
}

impl fmt::Display for TranslateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for TranslateError {}

pub type Result<T> = std::result::Result<T, TranslateError>;
