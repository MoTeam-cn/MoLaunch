//! 面板持有守卫：多批次串行操作期间持有面板显示，离开作用域时自动释放

use super::DownloadManager;

/// 面板持有守卫：多批次串行操作期间持有面板显示，离开作用域时（含错误提前返回）自动释放
pub struct PanelLease<'a> {
    manager: &'a DownloadManager,
}

impl<'a> PanelLease<'a> {
    /// 获取面板持有（立即 emit 显示，若为首个持有者）
    pub fn acquire(manager: &'a DownloadManager) -> Self {
        manager.hold_panel();
        Self { manager }
    }
}

impl Drop for PanelLease<'_> {
    fn drop(&mut self) {
        self.manager.release_panel();
    }
}
