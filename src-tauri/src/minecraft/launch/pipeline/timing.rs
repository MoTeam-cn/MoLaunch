//! 启动阶段耗时统计
//!
//! 每个阶段用 `StageClock` 记录起止，启动完成后由 `launch-timing` 事件下发给前端。

use serde::Serialize;
use std::time::Instant;

/// 单个阶段的耗时
#[derive(Debug, Clone, Serialize)]
pub struct StageTiming {
    /// 阶段标识（与 `LaunchStage` 的 Debug 名一致，前端据此映射中文）
    pub stage: String,
    /// 耗时（毫秒）
    pub ms: u64,
}

/// 一次启动的耗时明细
#[derive(Debug, Clone, Serialize)]
pub struct LaunchTimings {
    /// 各阶段耗时（按发生顺序）
    pub stages: Vec<StageTiming>,
    /// 各阶段耗时之和（毫秒）
    pub total_ms: u64,
    /// 版本 ID
    pub version_id: String,
}

/// 阶段计时器：构造即开始计时
pub struct StageClock(Instant);

impl StageClock {
    pub fn start() -> Self {
        Self(Instant::now())
    }

    /// 从计时开始到现在的毫秒数
    pub fn elapsed_ms(&self) -> u64 {
        self.0.elapsed().as_millis() as u64
    }
}
