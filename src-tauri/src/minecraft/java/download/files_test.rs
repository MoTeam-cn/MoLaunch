//! runtime 目录清理单测：共用的 runtime 绝不能被整删

use super::{
    cleanup_failed_install, clear_install_marker, is_install_complete, mark_install_started,
};

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

#[test]
fn test_install_marker_安装期间视为未装完() {
    let dir = temp_dir("marker");
    assert!(is_install_complete(&dir), "没有标记就当作装好了");

    mark_install_started(&dir);
    assert!(
        !is_install_complete(&dir),
        "装到一半的 java.exe 能跑但缺模块，不能被当成可用 Java"
    );

    clear_install_marker(&dir);
    assert!(is_install_complete(&dir), "装好之后要摘掉标记");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_install_marker_目录不存在时会被建出来() {
    let dir =
        std::env::temp_dir().join(format!("molaunch-javaruntime-{}-fresh", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    mark_install_started(&dir);

    assert!(!is_install_complete(&dir), "标记必须写下来");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_cleanup_failed_install_保留目录时摘掉标记() {
    let dir = temp_dir("marker-keep");
    mark_install_started(&dir);

    cleanup_failed_install(&dir, true);

    assert!(
        is_install_complete(&dir),
        "保留目录就必须摘标记，否则扫描器会永久忽略这个 runtime"
    );
    assert!(dir.join("bin").join("java.exe").exists());
    let _ = std::fs::remove_dir_all(&dir);
}
