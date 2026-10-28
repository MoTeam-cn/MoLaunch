//! 社区模块公共工具函数
//!
//! 供 curseforge / modrinth / preload / sources 等模块复用。

use std::time::{Duration, Instant};

/// 元数据请求的双线路对冲延迟
///
/// 官方线路在这个时间内没返回，就并发打镜像线路，先成功者胜出。取 2s：
/// 正常官方响应通常在数百毫秒内，2s 已能判定「这条线路慢了」。
pub(crate) const HEDGE_DELAY: Duration = Duration::from_secs(2);

/// 格式化耗时：< 1000ms 显示 ms，>= 1000ms 显示 s
///
/// 统一替代此前分散在 sources.rs / curseforge.rs / modrinth.rs / preload.rs 中的
/// 同名私有实现。
pub fn fmt_elapsed(start: Instant) -> String {
    let ms = start.elapsed().as_millis();
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}

/// 简单 URL 编码参数列表
///
/// 统一替代此前分散在 curseforge.rs / modrinth.rs 中的同名私有实现。
pub fn urlencode_params(params: &[(&str, String)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}
