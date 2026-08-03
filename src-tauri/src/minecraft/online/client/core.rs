//! OnlineClient 核心结构定义与基础访问器

/// api-server 客户端
pub struct OnlineClient {
    pub(super) base_url: String,
}

impl OnlineClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// 更新 base_url（用户在设置页修改 api-server 地址后调用）
    pub fn update_base_url(&mut self, base_url: &str) {
        self.base_url = base_url.trim_end_matches('/').to_string();
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
