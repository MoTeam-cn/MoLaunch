//! Forge 安装器参数与输出解码测试

use super::forge_installer::{
    build_installer_args, build_official_args, classpath_separator, decode_output_line,
};

#[test]
fn test_decode_output_line_passes_through_utf8() {
    assert_eq!(
        decode_output_line(b"Extracting json\r\n"),
        "Extracting json"
    );
}

#[test]
fn test_decode_output_line_falls_back_to_gbk() {
    // 「错误」的 GBK 编码，按 UTF-8 解码会失败
    let gbk = [0xB4, 0xED, 0xCE, 0xF3];
    assert!(std::str::from_utf8(&gbk).is_err());
    assert_eq!(decode_output_line(&gbk), "错误");
}

#[test]
fn test_classpath_separator_matches_platform() {
    let expected = if cfg!(windows) { ";" } else { ":" };
    assert_eq!(classpath_separator(), expected);
}

#[test]
fn test_build_args_without_wrapper() {
    let args = build_installer_args(8, "a.jar;b.jar", "w.jar", "/game", "/tmp", false);
    assert_eq!(
        args,
        vec![
            "-cp",
            "a.jar;b.jar",
            "com.bangbang93.ForgeInstaller",
            "/game"
        ]
    );
}

#[test]
fn test_build_args_with_wrapper_uses_property_equals_form() {
    let args = build_installer_args(8, "a.jar;b.jar", "w.jar", "/游戏 目录", "/tmp", true);
    // 关键：-Doolloo.jlw.tmpdir 必须和值写在同一个参数里，否则 JVM 会把值当成主类名
    assert_eq!(args[0], "-Doolloo.jlw.tmpdir=/tmp");
    assert!(!args.iter().any(|a| a == "-Doolloo.jlw.tmpdir"));
    assert_eq!(args[args.len() - 1], "/游戏 目录");
    assert!(args.iter().any(|a| a == "-jar"));
}

#[test]
fn test_build_args_adds_exports_for_java_9_and_above() {
    let args = build_installer_args(17, "a.jar", "w.jar", "/game", "/tmp", false);
    assert_eq!(args[0], "--add-exports");
    assert_eq!(
        args[1],
        "cpw.mods.bootstraplauncher/cpw.mods.bootstraplauncher=ALL-UNNAMED"
    );
}

#[test]
fn test_build_official_args_uses_install_client() {
    let args = build_official_args("C:/tmp/forge-installer.jar");
    assert_eq!(args[0], "-jar");
    assert_eq!(args[1], "C:/tmp/forge-installer.jar");
    assert_eq!(args[2], "--installClient");
}
