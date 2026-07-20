//! Java 版本权重模块

/// Java 版本权重
pub fn get_java_version_weight(major_version: u32) -> u32 {
    match major_version {
        7 => 0,
        8 => 30, // Java 8 权重最高（兼容性最好）
        9 => 4,
        10 => 5,
        11 => 14,
        12 => 6,
        13 => 7,
        14 => 8,
        15 => 9,
        16 => 12,
        17 => 31, // Java 17 权重最高（新版本推荐）
        18 => 13,
        19 => 10,
        20 => 11,
        21 => 29, // Java 21 权重高（最新 LTS）
        _ => major_version,
    }
}
