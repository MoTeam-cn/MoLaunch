//! Java 下载相关常量

/// Mojang Java Runtime 索引 URL（官方）
pub const JAVA_RUNTIME_INDEX_OFFICIAL: &str =
    "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

/// 文件下载域名替换：Mojang 官方域名 → BMCLAPI
pub const DOWNLOAD_DOMAIN_REPLACEMENTS: &[(&str, &str)] = &[
    ("https://piston-data.mojang.com", "https://bmclapi2.bangbang93.com"),
    ("https://piston-meta.mojang.com", "https://bmclapi2.bangbang93.com"),
];

/// Java 下载进度事件名
pub const JAVA_DOWNLOAD_PROGRESS_EVENT: &str = "java-download-progress";
