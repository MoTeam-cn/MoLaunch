//! 红石内核获取模块单元测试（平台命名、可执行体校验、endpoint 无关逻辑）

use super::{is_expected_executable, kernel_file_name, platform_arch};

/// 当前平台必须能解析出官方命名（本仓库只在受支持平台构建）
#[test]
fn test_platform_arch_当前平台可解析() {
    let (platform, arch) = platform_arch().expect("当前平台应有官方命名");
    assert!(
        matches!(platform, "windows" | "linux" | "macos"),
        "platform 取值非法: {platform}"
    );
    assert!(matches!(arch, "amd64" | "arm64"), "arch 取值非法: {arch}");
}

/// 文件名遵循官方 `{binary}-{platform}-{arch}`，仅 windows 带 .exe
#[test]
fn test_kernel_file_name_命名规则() {
    let name = kernel_file_name().expect("当前平台应有内核文件名");
    assert!(name.starts_with("hongshic-"), "应以 hongshic- 开头: {name}");
    #[cfg(target_os = "windows")]
    assert!(name.ends_with(".exe"), "windows 应以 .exe 结尾: {name}");
    #[cfg(not(target_os = "windows"))]
    assert!(!name.ends_with(".exe"), "非 windows 不应带 .exe: {name}");
}

/// PE / ELF / Mach-O 魔数识别
#[test]
fn test_is_expected_executable_各平台魔数() {
    assert!(is_expected_executable(b"MZ\x90\x00", "windows"));
    assert!(!is_expected_executable(
        b"{\"detail\":\"Not Found\"}",
        "windows"
    ));
    assert!(is_expected_executable(b"\x7fELF\x02\x01", "linux"));
    assert!(!is_expected_executable(b"<html>", "linux"));
    assert!(is_expected_executable(
        &[0xcf, 0xfa, 0xed, 0xfe, 0x07],
        "macos"
    ));
    assert!(is_expected_executable(
        &[0xfe, 0xed, 0xfa, 0xcf, 0x00],
        "macos"
    ));
    assert!(!is_expected_executable(b"MZ\x90\x00", "macos"));
}

/// 未知平台一律判否，避免把错误体写进缓存
#[test]
fn test_is_expected_executable_未知平台判否() {
    assert!(!is_expected_executable(b"MZ\x90\x00", "android"));
}

/// 空内容判否（下载到 0 字节时不写缓存）
#[test]
fn test_is_expected_executable_空内容判否() {
    assert!(!is_expected_executable(b"", "windows"));
    assert!(!is_expected_executable(b"", "linux"));
    assert!(!is_expected_executable(b"", "macos"));
}
