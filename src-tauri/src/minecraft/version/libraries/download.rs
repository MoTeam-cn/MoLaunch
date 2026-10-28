//! 下载 URL 构建：声明来源 + 镜像 + 老构件兜底链

use crate::minecraft::sources;

use super::LibEntry;

/// 构建候选下载 URL
///
/// 顺序：用户自配镜像（若有）→ 声明来源 → 兜底链（BMCLAPI → Mojang → 老构件原仓库
/// → Maven Central → 阿里云）。真正的最终顺序由 `reorder_urls` 按源模式重排，
/// 这里只保证同一族内部的优先级。
///
/// 没有声明 sha1 的构件同样走完整候选链：这是刻意的取舍——顺序上官方来源永远在前，
/// 镜像只在官方下不动时兜底；如果为了「无法证明来源」而砍掉镜像，老版本会直接下不来。
/// 哈希可用时（绝大多数构件）由 `FileChecker` 逐字节校验，镜像给的文件对不上会失败重下。
pub fn build_download_urls(lib: &LibEntry, mirror_url: Option<&str>) -> Vec<String> {
    let relative = sources::relative_artifact_path(lib.original_name.as_deref(), &lib.local_path);
    let mut urls = Vec::new();

    // 用户自配镜像优先：路径按构件相对路径拼，比从原始 URL 里截更可靠
    if let Some(mirror) = mirror_url.filter(|mirror| !mirror.is_empty()) {
        if !relative.is_empty() {
            sources::push_unique(
                &mut urls,
                format!("{}/{}", mirror.trim_end_matches('/'), relative),
            );
        }
    }

    if let Some(ref url) = lib.url {
        sources::push_unique(&mut urls, url.clone());
    }

    for candidate in sources::library_fallbacks(&relative) {
        sources::push_unique(&mut urls, candidate);
    }

    urls
}
