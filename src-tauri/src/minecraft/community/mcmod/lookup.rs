//! 公开查询函数：通过平台和 Slug 查找中文译名/class id

use super::super::types::Platform;
use super::database::DATABASE;

/// 通过 CurseForge Slug 查找中文译名
pub fn lookup_cf(slug: &str) -> Option<&str> {
    DATABASE.lookup_cf(slug).map(|e| e.chinese_name.as_str())
}

/// 通过 Modrinth Slug 查找中文译名
pub fn lookup_mr(slug: &str) -> Option<&str> {
    DATABASE.lookup_mr(slug).map(|e| e.chinese_name.as_str())
}

/// 通过平台和 Slug 查找中文译名
pub fn translate(platform: Platform, slug: &str) -> Option<&str> {
    match platform {
        Platform::CurseForge => lookup_cf(slug),
        Platform::Modrinth => lookup_mr(slug),
    }
}

/// 通过平台和 Slug 查找 MC 百科 class id（用于拼接详情页 URL）
pub fn lookup_class_id(platform: Platform, slug: &str) -> Option<u32> {
    match platform {
        Platform::CurseForge => DATABASE.lookup_cf(slug).map(|e| e.class_id),
        Platform::Modrinth => DATABASE.lookup_mr(slug).map(|e| e.class_id),
    }
}
