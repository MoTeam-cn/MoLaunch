//! 退役镜像地址归一化
//!
//! 老整合包 manifest、老教程里保存的镜像域名早已下线，照原样下载必然 404。
//! 这里按「主机 + 路径前缀」把它们还原成官方地址，避免整包安装卡在几条死链上。
//! 只处理能 1:1 对上官方路径的情况，对不上的一律原样返回（不猜、不拼）。

use std::borrow::Cow;

/// 一条还原规则
struct Rule {
    /// 退役主机
    host: &'static str,
    /// 路径前缀（必须以它开头才还原）
    prefix: &'static str,
    /// 目标官方主机
    target: &'static str,
    /// 从前缀后剪掉的字节数（老路径多出来的那一段）
    strip: usize,
}

const RULES: &[Rule] = &[
    // 备用 CDN：路径与官方完全一致
    Rule {
        host: "cdn-alt.modrinth.com",
        prefix: "/",
        target: "cdn.modrinth.com",
        strip: 0,
    },
    // 旧第三方镜像：按路径前缀分别还原到 Modrinth / CurseForge 官方
    Rule {
        host: "mod.tianpao.top",
        prefix: "/data/",
        target: "cdn.modrinth.com",
        strip: 0,
    },
    Rule {
        host: "mod.tianpao.top",
        prefix: "/files/",
        target: "edge.forgecdn.net",
        strip: 0,
    },
    // /media/attachments/x -> media.forgecdn.net/attachments/x
    Rule {
        host: "mod.tianpao.top",
        prefix: "/media/",
        target: "media.forgecdn.net",
        strip: 6,
    },
];

/// 把退役镜像地址还原成官方地址（无法还原时原样返回）
pub fn normalize(url: &str) -> Cow<'_, str> {
    let Some(rest) = url.strip_prefix("https://") else {
        return Cow::Borrowed(url);
    };
    let split = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..split];
    let suffix = &rest[split..];
    let host = authority.split(':').next().unwrap_or(authority);

    let path_end = suffix.find(['?', '#']).unwrap_or(suffix.len());
    let (path, tail) = suffix.split_at(path_end);

    for rule in RULES {
        if !host.eq_ignore_ascii_case(rule.host) || !path.starts_with(rule.prefix) {
            continue;
        }
        // strip 一定落在 ASCII 前缀之后；用 get() 兜底，越界或落在多字节字符中间就原样返回
        let Some(remainder) = path.get(rule.strip..) else {
            continue;
        };
        return Cow::Owned(format!("https://{}{}{}", rule.target, remainder, tail));
    }
    Cow::Borrowed(url)
}

#[cfg(test)]
#[path = "retired_test.rs"]
mod retired_test;
