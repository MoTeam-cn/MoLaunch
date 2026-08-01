//! yggdrasil 客户端的错误类型与 authlib-injector 元数据类型

/// yggdrasil API 错误（含 HTTP 状态码与解析后的错误消息）
#[derive(Debug)]
pub struct YggdrasilError {
    pub status: u16,
    pub message: String,
    /// 是否为网络错误（true 表示无法连接服务器，可重试）
    pub is_network: bool,
}

impl std::fmt::Display for YggdrasilError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_network {
            write!(f, "无法连接到服务器（网络错误）: {}", self.message)
        } else {
            write!(f, "服务器返回错误 ({}): {}", self.status, self.message)
        }
    }
}

impl From<YggdrasilError> for String {
    fn from(e: YggdrasilError) -> String {
        e.to_string()
    }
}

/// authlib-injector.jar 下载元数据
#[derive(Debug, Clone)]
pub struct AuthlibInjectorMeta {
    /// 官方下载地址
    pub download_url: String,
    /// sha256 校验值
    pub sha256: String,
}
