//! 下载失败码
//!
//! 前端拿到 `error_code` 要给用户一句能看懂的话，所以这些数字必须集中定义：
//! 散落在各个 `mark_failed(1)` 里的魔法数字改不动也读不懂。
//!
//! 0 表示「没有错误」，非 0 一律表示失败。

/// 没有错误
pub const NONE: i32 = 0;

/// 网络/下载本身失败：所有源都试过仍拿不到数据
pub const NETWORK: i32 = 1;

/// 校验失败：大小、哈希或内容对不上
pub const VERIFY: i32 = 2;

/// 磁盘/文件系统问题：空间不足、权限、路径被占用
pub const DISK: i32 = 3;

/// 用户取消
pub const CANCELLED: i32 = 4;

/// 安装流程失败：文件下完了，但后续步骤（解压、写配置、装加载器）出错
pub const INSTALL: i32 = 5;

#[cfg(test)]
#[path = "error_code_test.rs"]
mod error_code_test;
