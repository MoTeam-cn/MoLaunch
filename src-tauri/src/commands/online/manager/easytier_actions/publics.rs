//! easytier 默认公共节点（信令 / 中继）。
//!
//! 内置项目自建信令节点；创建 / 加入虚拟网络组装 `--peers` 时兜底追加，
//! 保证组网必有可用信令节点。前端设置页不展示，用户仅管理自定义节点。

/// 项目自建 easytier 信令节点（不支持中继）
pub(crate) const DEFAULT_SIGNALING_PEER: &str = "wss://node1.molaunch.moiu.cn";

/// 默认公共节点列表（后续新增节点在此追加）
pub(crate) fn default_peers() -> Vec<&'static str> {
    vec![DEFAULT_SIGNALING_PEER]
}
