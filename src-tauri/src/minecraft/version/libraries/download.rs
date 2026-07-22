//! 下载 URL 构建：镜像替换 + fallback URL 列表生成

use crate::minecraft::sources;

use super::LibEntry;

/// Build download URLs
pub fn build_download_urls(lib: &LibEntry, mirror_url: Option<&str>) -> Vec<String> {
    let mut urls = Vec::new();

    if let Some(ref url) = lib.url {
        urls.push(url.clone());

        // BMCLAPI/maven 替换
        let bmclapi_url = sources::apply_replacements(url, sources::MAVEN_REPLACEMENTS);

        if bmclapi_url != *url {
            urls.push(bmclapi_url.clone());

            // BMCLAPI/libraries 替换
            let bmclapi_lib_url = sources::apply_replacements(url, sources::LIBRARY_REPLACEMENTS);

            if bmclapi_lib_url != *url && bmclapi_lib_url != bmclapi_url {
                urls.push(bmclapi_lib_url);
            }
        }
    }

    if urls.is_empty() {
        let relative = lib
            .local_path
            .replace("\\", "/")
            .split("/libraries/")
            .last()
            .unwrap_or("")
            .to_string();

        if !relative.is_empty() {
            urls.push(format!("{}/{}", sources::MOJANG_LIBRARIES, relative));
            urls.push(format!("{}/maven/{}", sources::BMCLAPI_BASE, relative));
        }
    }

    if let Some(mirror) = mirror_url {
        let mirror_base = mirror.trim_end_matches('/');
        if let Some(ref url) = lib.url {
            let mirror_url = format!(
                "{}/{}",
                mirror_base,
                url.split("/maven/")
                    .last()
                    .or_else(|| url.split("/libraries/").last())
                    .unwrap_or("")
            );
            if !urls.contains(&mirror_url) {
                urls.insert(0, mirror_url);
            }
        }
    }

    urls
}
