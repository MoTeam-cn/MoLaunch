//! 跨平台 shell 命令封装入口
//! 统一管理系统级外部命令调用，避免业务代码直接操作 `std::process::Command`。
//! 子模块：open（文件/URL 打开）、exec（命令执行/进程管理）、perms（文件权限）、
//! admin（管理员权限）、window（macOS/Linux 窗口管理命令）。

mod admin;
mod exec;
mod open;
mod perms;
mod window;

pub use admin::{is_admin, relaunch_as_admin};
pub use exec::{kill_process_tree, run_executable_output};
pub use open::{open_path, open_url, reveal_in_file_manager};
pub use perms::{restrict_dir_permissions, restrict_file_permissions};

#[cfg(target_os = "macos")]
pub use window::run_osascript;

#[cfg(target_os = "linux")]
pub use window::{
    ps_pid_exists, wmctrl_list, wmctrl_rename, xdotool_search_pid, xdotool_set_window_name,
};
