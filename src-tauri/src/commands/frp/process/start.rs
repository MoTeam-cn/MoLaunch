//! frpc 启动：校验二进制 → 生成配置 → 启动子进程 → 捕获日志 → 监听退出。

use super::{monitor, spawn};
use crate::commands::frp::process::RUNNING;
use crate::commands::frp::tunnel;
use crate::log_info;
use crate::state::AppState;
use tauri::AppHandle;

/// 启动隧道。
pub async fn start_tunnel(state: &AppState, id: String, app: AppHandle) -> Result<(), String> {
    {
        let running = RUNNING.lock().await;
        if running.contains_key(&id) {
            return Err(format!("隧道已在运行: {}", id));
        }
    }

    let tunnel = tunnel::list_tunnels()
        .await?
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| format!("隧道不存在: {}", id))?;
    let spawned = spawn::spawn_frpc(state, tunnel.clone()).await?;
    let pid = monitor::register_and_monitor(spawned, app).await?;
    log_info!("[Frp] 隧道已启动: {} (PID {})", tunnel.name, pid);
    Ok(())
}

#[cfg(target_os = "windows")]
pub(super) fn assign_process_to_job_object(pid: u32) -> Result<(), String> {
    use std::ffi::c_void;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE};
    unsafe {
        let job =
            CreateJobObjectW(None, None).map_err(|e| format!("创建 Job Object 失败: {}", e))?;
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
        .map_err(|e| format!("设置 Job Object 信息失败: {}", e))?;
        let process = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, false, pid)
            .map_err(|e| format!("打开 frpc 进程失败: {}", e))?;
        AssignProcessToJobObject(job, process)
            .map_err(|e| format!("关联进程到 Job Object 失败: {}", e))?;
        let _ = CloseHandle(process);
    }
    Ok(())
}
