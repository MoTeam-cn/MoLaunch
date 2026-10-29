//! runtime 目录清理单测：共用的 runtime 绝不能被整删

use super::cleanup_failed_install;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "molaunch-javaruntime-{}-{}",
        tag,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).expect("建 runtime 目录");
    std::fs::write(dir.join("bin").join("java.exe"), b"stub").expect("写占位文件");
    dir
}

#[test]
fn test_cleanup_failed_install_已有目录保留不删() {
    let dir = temp_dir("keep");
    cleanup_failed_install(&dir, true);
    assert!(
        dir.join("bin").join("java.exe").exists(),
        "runtime 与官方启动器共用，不是本次新建的就不能删"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_cleanup_failed_install_本次新建才整删() {
    let dir = temp_dir("remove");
    cleanup_failed_install(&dir, false);
    assert!(!dir.exists(), "本次新建的目录装失败就该清掉");
}
