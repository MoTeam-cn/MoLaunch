//! 启动历史记录

use serde::{Deserialize, Serialize};

/// 启动历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchHistory {
    /// 版本ID
    pub version_id: String,
    /// 用户名
    pub username: String,
    /// 启动时间
    pub launch_time: String,
    /// 进程ID
    pub pid: u32,
    /// 退出码（如果有）
    pub exit_code: Option<i32>,
}
