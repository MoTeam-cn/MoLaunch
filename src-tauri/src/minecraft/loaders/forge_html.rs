//! Forge HTML 解析模块
//!
//! 解析 Forge 官方网站的 HTML 版本列表页面。

use super::LoaderVersion;
use super::utils;

/// 解析 Forge 官方 HTML 页面，提取版本列表
///
/// HTML 结构示例：
/// ```html
/// <tr>
///   <td class="download-version">
///     64.0.11
///     <i class="promo-latest fa"></i>
///   </td>
///   <td class="download-time" title="2026-06-28 13:28:32">2026-06-28</td>
///   ...
/// </tr>
/// ```
pub fn parse_forge_version_html(html: &str) -> anyhow::Result<Vec<LoaderVersion>> {
    let mut versions = Vec::new();

    for row in html.split("<tr>") {
        let version_td = match row.find(r#"<td class="download-version""#) {
            Some(idx) => &row[idx..],
            None => continue,
        };

        // 提取版本号
        let version = match extract_version_number(version_td) {
            Some(v) => v,
            None => continue,
        };

        // 推荐/最新版本标记
        let is_recommended = row.contains("promo-recommended") || row.contains("promo-latest");

        // 发布时间
        let release_time = extract_release_time(row);

        versions.push(LoaderVersion {
            version,
            is_recommended,
            release_time,
        });
    }

    // 按版本号降序排列
    versions.sort_by(|a, b| {
        let v_a = utils::parse_version_number(&a.version);
        let v_b = utils::parse_version_number(&b.version);
        v_b.cmp(&v_a)
    });

    Ok(versions)
}

/// 从 download-version td 中提取版本号
fn extract_version_number(td_content: &str) -> Option<String> {
    // 跳过 td 标签本身，找到 '>' 后的内容
    let after_tag = &td_content[td_content.find('>').map(|i| i + 1).unwrap_or(0)..];

    let mut version = String::new();
    for c in after_tag.chars() {
        if c.is_ascii_digit() || (c == '.' && !version.is_empty()) {
            version.push(c);
        } else if !version.is_empty() {
            break;
        }
    }

    if version.is_empty() { None } else { Some(version) }
}

/// 从表格行中提取发布时间
///
/// 查找 `<td class="download-time" title="2026-06-28 13:28:32">` 格式
/// 时间为 UTC，需转换为本地时间
fn extract_release_time(row: &str) -> Option<String> {
    let marker = r#"class="download-time" title=""#;
    let start_idx = row.find(marker)?;
    let after = &row[start_idx + marker.len()..];
    let end = after.find('"')?;
    let time_str = &after[..end];

    // 使用公共函数转换 UTC -> 本地时间
    utils::parse_utc_to_local(time_str)
}
