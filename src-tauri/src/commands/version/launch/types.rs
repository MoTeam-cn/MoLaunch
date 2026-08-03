//! 版本启动共享类型

/// 游戏退出事件数据
#[derive(Clone, serde::Serialize)]
pub struct GameExitEvent {
    pub pid: u32,
    pub version_id: String,
    pub exit_code: i32,
    pub is_normal: bool,
    /// 崩溃详情（仅异常退出时可能有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash_info: Option<crate::minecraft::launch::watcher::CrashInfo>,
}