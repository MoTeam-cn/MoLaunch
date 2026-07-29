use std::path::Path;
use std::process::Command;
use std::time::Duration;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING};
use windows::Win32::System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE};

pub fn wait_for_process_exit(pid: u32, timeout: Duration) -> bool {
    unsafe {
        let handle: HANDLE = match OpenProcess(PROCESS_SYNCHRONIZE, false, pid) {
            Ok(h) => h,
            Err(_) => return true,
        };
        let result = WaitForSingleObject(handle, timeout.as_millis() as u32);
        let _ = CloseHandle(handle);
        result == WAIT_OBJECT_0
    }
}

pub fn replace_exe(old_exe: &Path, new_exe: &Path) -> Result<(), String> {
    if try_move_replace(new_exe, old_exe) {
        return Ok(());
    }

    let backup = old_exe.with_extension("exe.old");
    std::fs::rename(old_exe, &backup).map_err(|e| format!("备份旧 exe 失败: {e}"))?;

    if let Err(e) = std::fs::rename(new_exe, old_exe) {
        let _ = std::fs::rename(&backup, old_exe);
        return Err(format!("移动新 exe 失败: {e}"));
    }
    Ok(())
}

fn try_move_replace(src: &Path, dst: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;

    let src_wide: Vec<u16> = src.as_os_str().encode_wide().chain(Some(0)).collect();
    let dst_wide: Vec<u16> = dst.as_os_str().encode_wide().chain(Some(0)).collect();

    unsafe {
        MoveFileExW(
            PCWSTR(src_wide.as_ptr()),
            PCWSTR(dst_wide.as_ptr()),
            MOVEFILE_REPLACE_EXISTING,
        )
        .is_ok()
    }
}

pub fn launch_new_exe(exe_path: &Path) -> Result<(), String> {
    Command::new(exe_path)
        .spawn()
        .map_err(|e| format!("启动新 exe 失败: {e}"))?;
    Ok(())
}
