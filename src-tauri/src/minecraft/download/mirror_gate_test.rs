//! 镜像闸门单测：退避序列、主机归并、镜像识别与冷却查询

use super::{backoff_for, cooldown_remaining, gate_key, is_mirror, BACKOFF_BASE, BACKOFF_MAX};
use std::time::Duration;

/// 退避呈指数增长并在上限处封顶
#[test]
fn test_backoff_for_指数增长并封顶() {
    assert_eq!(backoff_for(1), BACKOFF_BASE);
    assert_eq!(backoff_for(2), Duration::from_secs(4));
    assert_eq!(backoff_for(3), Duration::from_secs(8));
    assert_eq!(backoff_for(4), Duration::from_secs(16));
    // 第 5 步起 32s 超过上限，按 30s 封顶
    assert_eq!(backoff_for(5), BACKOFF_MAX);
    assert_eq!(backoff_for(9), BACKOFF_MAX);
    // 0 与非法步数按首次处理，不 panic
    assert_eq!(backoff_for(0), BACKOFF_BASE);
}

/// 同一镜像主机的不同路径必须归并到同一 key（共享槽位与冷却）
#[test]
fn test_gate_key_按主机归并() {
    assert_eq!(
        gate_key("https://bmclapi2.bangbang93.com/maven/net/fabricmc/x.jar"),
        "bmclapi2.bangbang93.com"
    );
    assert_eq!(
        gate_key("https://bmclapi2.bangbang93.com/assets/ab/abcdef"),
        gate_key("https://bmclapi2.bangbang93.com/libraries/x.jar")
    );
    // 无法解析的 URL 退化为整条字符串，避免误并到同一桶
    assert_eq!(gate_key("not a url"), "not a url");
}

/// 只按主机名识别公共镜像：路径里含关键字的普通站点不参与限流
#[test]
fn test_is_mirror_按主机名识别() {
    assert!(is_mirror("https://bmclapi2.bangbang93.com/assets/ab/x"));
    assert!(is_mirror("https://mod.mcimirror.top/modrinth/v2/search"));
    assert!(is_mirror("https://cdn-modrinth.mocdn.net/data/x/file.jar"));
    // 自定义镜像主机同样参与限流（与 sources::is_mirror_url 的判定一致）
    assert!(is_mirror("https://bmclapi-mirror.example.com/file.jar"));
    assert!(!is_mirror("https://cdn.modrinth.com/data/x/file.jar"));
    // 只是路径里出现关键字，不是镜像域
    assert!(!is_mirror("https://example.com/bmclapi/path"));
    assert!(!is_mirror(""));
}

/// 非镜像 URL 查询冷却必须为零成本空操作
#[test]
fn test_cooldown_remaining_非镜像永远无冷却() {
    assert!(cooldown_remaining("https://cdn.modrinth.com/data/x/file.jar").is_none());
    assert!(cooldown_remaining("https://edge.forgecdn.net/files/1/2/x.jar").is_none());
}
