//! 外部厂商卸载职责。

use super::super::provider::{read_providers_state, write_providers_state, SYSTEM_DEFAULT_ID};
use super::super::{providers_root, validate_provider_id};
use crate::log_info;

pub async fn uninstall_provider(provider_id: String) -> Result<(), String> {
    if provider_id == SYSTEM_DEFAULT_ID {
        return Err("不能卸载系统默认厂商".to_string());
    }
    validate_provider_id(&provider_id)?;
    let dir = providers_root().join(&provider_id);
    if !dir.exists() {
        return Err(format!("厂商不存在: {}", provider_id));
    }
    let canonical_root = providers_root()
        .canonicalize()
        .map_err(|e| format!("路径校验失败: {}", e))?;
    let canonical_dir = dir
        .canonicalize()
        .map_err(|e| format!("路径校验失败: {}", e))?;
    if !canonical_dir.starts_with(&canonical_root) {
        return Err("路径遍历检测".to_string());
    }
    std::fs::remove_dir_all(&dir).map_err(|e| format!("卸载失败: {}", e))?;
    let mut state = read_providers_state();
    state.remove(&provider_id);
    write_providers_state(&state)?;
    log_info!("[Frp] 厂商已卸载: {}", provider_id);
    Ok(())
}
