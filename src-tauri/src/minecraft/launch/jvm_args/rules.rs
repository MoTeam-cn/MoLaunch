//! JVM 版本规则
//!
//! 按 Java 主版本选择 GC 策略与 GBK 编码检测。

/// GC 策略：Java 21+ 使用 ZGC + ZGenerational，Java 15+ 使用 ZGC，否则 G1GC
pub(super) fn add_gc_args(args: &mut Vec<String>, java_major: Option<u32>) {
    if let Some(version) = java_major {
        if version >= 21 {
            args.push("-XX:+UseZGC".to_string());
            args.push("-XX:+ZGenerational".to_string());
        } else if version >= 15 {
            args.push("-XX:+UseZGC".to_string());
        } else {
            args.push("-XX:+UseG1GC".to_string());
        }
    }
}

/// 检测系统是否使用 GBK 编码（Windows ANSI 代码页 936）
#[cfg(target_os = "windows")]
pub(super) fn is_gbk_encoding() -> bool {
    // 通过注册表读取系统 ANSI 代码页：HKLM\SYSTEM\CurrentControlSet\Control\Nls\CodePage::ACP
    // 936 = GBK，返回 true；其他值返回 false
    use winreg::enums::*;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\Nls\\CodePage") {
        if let Ok(acp) = key.get_value::<String, _>("ACP") {
            return acp == "936";
        }
    }
    // 读取失败时默认非 GBK（避免误触发 JLW）
    false
}

#[cfg(not(target_os = "windows"))]
pub(super) fn is_gbk_encoding() -> bool {
    false
}