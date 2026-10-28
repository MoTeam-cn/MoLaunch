//! 库文件候选链：一个 maven 构件该去哪些仓库找
//!
//! 顺序即优先级：声明来源 → BMCLAPI → Mojang Maven → 老构件原仓库 → Maven Central → 阿里云。
//! Mojang 的 Maven 这些年下架了不少老构件（`net.java.jinput` 平台 jar 直接 404），
//! Central 还留着，所以它排在兜底链的倒数第二位；阿里云只是 Central 的国内镜像。

use super::constants::{
    ALIYUN_MAVEN_PUBLIC, BMCLAPI_BASE, MAVEN_CENTRAL, MAVEN_FABRIC, MAVEN_FORGE, MAVEN_NEOFORGE,
    MAVEN_SPONGE, MOJANG_LIBRARIES,
};

/// 从 maven 坐标或本地路径推出构件相对路径
///
/// 坐标优先（带 classifier 也能算对）；拿不到坐标时退回本地路径里 `libraries/` 之后的部分。
pub fn relative_artifact_path(maven_name: Option<&str>, local_path: &str) -> String {
    if let Some(name) = maven_name.filter(|name| name.matches(':').count() >= 2) {
        return crate::minecraft::utils::maven::maven_to_relative_path(name);
    }
    // 只有确实是 `<game>/libraries/<相对路径>` 才敢截取，否则拼出来的候选地址都是垃圾
    let normalized = local_path.replace('\\', "/");
    match normalized.split_once("/libraries/") {
        Some((_, relative)) => relative.to_string(),
        None => String::new(),
    }
}

/// 按 groupId 前缀猜这类老构件原本在哪个仓库
///
/// 只列「声明来源与 BMCLAPI 都不一定有」的仓库；猜不出就返回空，交给后面的兜底。
pub fn legacy_repositories(relative: &str) -> &'static [&'static str] {
    const CENTRAL_GROUPS: &[&str] = &[
        "org/ow2/",
        "org/scala-lang/",
        "org/jline/",
        "jline/",
        "net/java/dev/jna/",
        "com/typesafe/",
    ];
    if CENTRAL_GROUPS
        .iter()
        .any(|prefix| relative.starts_with(prefix))
    {
        &[MAVEN_CENTRAL]
    } else if relative.starts_with("net/fabricmc/") {
        &[MAVEN_FABRIC]
    } else if relative.starts_with("net/minecraftforge/") || relative.starts_with("cpw/mods/") {
        &[MAVEN_FORGE]
    } else if relative.starts_with("net/neoforged/") {
        &[MAVEN_NEOFORGE]
    } else if relative.starts_with("org/spongepowered/") {
        &[MAVEN_SPONGE]
    } else {
        &[]
    }
}

/// 兜底候选链（不含调用方声明的 URL）
///
/// BMCLAPI 的 `/maven/` 与 `/libraries/` 都是按构件路径 1:1 代理，两个都试是刻意的：
/// 老版本 Forge 的构件在两个路径下的分布并不一致。
pub fn fallback_urls(relative: &str) -> Vec<String> {
    if relative.is_empty() {
        return Vec::new();
    }
    let mut urls = vec![
        format!("{BMCLAPI_BASE}/maven/{relative}"),
        format!("{BMCLAPI_BASE}/libraries/{relative}"),
        format!("{MOJANG_LIBRARIES}/{relative}"),
    ];
    for base in legacy_repositories(relative) {
        urls.push(format!("{base}/{relative}"));
    }
    urls.push(format!("{MAVEN_CENTRAL}/{relative}"));
    urls.push(format!("{ALIYUN_MAVEN_PUBLIC}/{relative}"));
    urls
}

/// 追加候选并去重（保序：先来的优先级更高）
pub fn push_unique(urls: &mut Vec<String>, url: String) {
    if !urls.contains(&url) {
        urls.push(url);
    }
}

#[cfg(test)]
#[path = "library_chain_test.rs"]
mod library_chain_test;
