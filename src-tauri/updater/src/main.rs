mod args;
mod log;
mod platform;
mod verify;

use std::time::Duration;

use args::Args;
use log::log;
use platform::{launch_new_exe, replace_exe, wait_for_process_exit};
use verify::verify_minisign;

const WAIT_TIMEOUT_SECS: u64 = 30;

fn main() {
    let args = match Args::parse() {
        Ok(a) => a,
        Err(e) => {
            log(&format!("参数解析失败: {e}"));
            std::process::exit(1);
        }
    };

    log(&format!(
        "updater 启动: old={}, new={}, pid={}",
        args.old_exe.display(),
        args.new_exe.display(),
        args.pid
    ));

    if !wait_for_process_exit(args.pid, Duration::from_secs(WAIT_TIMEOUT_SECS)) {
        log(&format!("等待主进程 {} 退出超时（{}s）", args.pid, WAIT_TIMEOUT_SECS));
        std::process::exit(2);
    }
    log("主进程已退出");

    if let Err(e) = verify_minisign(&args.new_exe, &args.signature) {
        log(&format!("签名校验失败: {e}"));
        std::process::exit(5);
    }
    log("签名校验通过");

    if let Err(e) = replace_exe(&args.old_exe, &args.new_exe) {
        log(&format!("替换 exe 失败: {e}"));
        std::process::exit(3);
    }
    log("exe 替换成功");

    if let Err(e) = launch_new_exe(&args.old_exe) {
        log(&format!("启动新 exe 失败: {e}"));
        std::process::exit(4);
    }
    log("新 exe 已启动，updater 退出");
}
