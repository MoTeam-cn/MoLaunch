use super::super::super::types::LogLevel;
use super::*;

fn log(level: LogLevel, msg: &str) -> LogEntry {
    LogEntry {
        timestamp: String::new(),
        level,
        source: String::new(),
        message: msg.to_string(),
    }
}

/// 同步运行异步分析（测试用）
fn block_on<T>(fut: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(fut)
}

/// 运行分析（同步测试用）；使用不存在的目录以保证不读取到真实崩溃文件
fn run(logs: &[LogEntry], game_dir: &Path) -> Option<CrashInfo> {
    block_on(analyze_crash(1, logs, game_dir))
}

fn test_dir() -> &'static Path {
    Path::new("./crash_analyzer_test_nonexistent_dir")
}

#[test]
fn exit_code_zero_returns_none() {
    let logs = [log(LogLevel::Info, "hello")];
    let info = block_on(analyze_crash(0, &logs, test_dir()));
    assert!(info.is_none());
}

#[test]
fn detects_unrecognized_option() {
    let logs = [
        log(LogLevel::Error, "Unrecognized option: -Xfoo"),
        log(
            LogLevel::Error,
            "Error: Could not create the Java Virtual Machine",
        ),
    ];
    let info = run(&logs, test_dir()).unwrap();
    assert_eq!(info.category, CrashCategory::Java);
    assert!(info.reason.contains("无法识别的选项"));
}

#[test]
fn detects_out_of_memory() {
    let logs = [log(
        LogLevel::Error,
        "java.lang.OutOfMemoryError: Java heap space",
    )];
    let info = run(&logs, test_dir()).unwrap();
    assert_eq!(info.category, CrashCategory::Memory);
}

#[test]
fn fallback_unknown_on_no_hit() {
    let logs = [log(
        LogLevel::Error,
        "some unrelated fatal text that is long enough to avoid the short output detector triggering, \\
         since short outputs are handled separately in the analyzer pipeline",
    )];
    let info = run(&logs, test_dir()).unwrap();
    assert_eq!(info.category, CrashCategory::Unknown);
    assert!(info.reason.contains("退出码"));
}
