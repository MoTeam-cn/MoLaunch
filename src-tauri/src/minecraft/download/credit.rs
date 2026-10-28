//! 进度记账：一个文件的「已计入全局进度的字节」由它独占管理
//!
//! 下载路径上有好几处都想把进度往前推（引擎每个采样窗口、恢复位图后的既有字节、
//! 重跑后重新下载的块）。如果各自 push 自己的增量，就会重复累计——之前块级写盘与
//! 引擎按轮同步各算一遍，一个 100 MB 的文件会被记成 200 MB。
//!
//! 这里把接口统一成「设置当前累计值」，内部只按差额增减：调用多少次、谁来调用都幂等。

use std::sync::{Arc, Mutex as StdMutex};

use super::types::GlobalProgress;

/// 单个文件的进度记账句柄
pub(crate) struct ProgressCredit<'a> {
    progress: &'a Option<Arc<StdMutex<GlobalProgress>>>,
    counted: StdMutex<u64>,
}

impl<'a> ProgressCredit<'a> {
    pub(crate) fn new(progress: &'a Option<Arc<StdMutex<GlobalProgress>>>) -> Self {
        Self {
            progress,
            counted: StdMutex::new(0),
        }
    }

    /// 把本文件对全局进度的累计贡献设置成 `desired`
    ///
    /// 只按差额增减：`desired` 比已记账的大就补上，小就退回（作废重下时会变小）。
    pub(crate) fn credit(&self, desired: u64) {
        let mut counted = self
            .counted
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = *counted;
        if desired == previous {
            return;
        }
        *counted = desired;
        drop(counted);
        let Some(progress) = self.progress else {
            return;
        };
        let mut guard = progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if desired > previous {
            guard.downloaded_bytes = guard.downloaded_bytes.saturating_add(desired - previous);
        } else {
            guard.downloaded_bytes = guard.downloaded_bytes.saturating_sub(previous - desired);
        }
    }

    /// 全部退回（本文件最终失败、由别的路径接手时调用）
    pub(crate) fn rollback(&self) {
        self.credit(0);
    }
}

#[cfg(test)]
#[path = "credit_test.rs"]
mod credit_test;
