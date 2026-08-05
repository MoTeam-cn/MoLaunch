//! 本地 AI 服务配置（OpenAI 兼容 API）

use serde::{Deserialize, Serialize};

/// OpenAI 兼容 API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    /// 服务地址，如 http://127.0.0.1:11434/v1
    pub base_url: String,
    /// API Key（明文，保存时经 SDK DES 加密后写入 config.ini）
    pub api_key: String,
    /// 超时秒数
    pub timeout_secs: u64,
    /// 已启用（导入）的模型列表
    #[serde(default)]
    pub models: Vec<String>,
    /// 默认模型名
    #[serde(default)]
    pub default_model: String,
}

impl AiConfig {
    /// 解析实际使用的模型：显式指定 > 默认模型 > 已启用模型首个
    pub fn resolve_model(&self, preferred: Option<&str>) -> String {
        if let Some(m) = preferred.filter(|s| !s.trim().is_empty()) {
            return m.trim().to_string();
        }
        if !self.default_model.trim().is_empty() {
            return self.default_model.clone();
        }
        self.models.first().cloned().unwrap_or_default()
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            // 默认留空：需在设置页配置本地服务地址与模型后才能使用 AI 功能
            base_url: String::new(),
            api_key: String::new(),
            timeout_secs: 60,
            models: Vec::new(),
            default_model: String::new(),
        }
    }
}
