//! Windows deeplink 协议实现（HKCU 注册表，免管理员）

use super::PROTOCOL;

/// 注册表根键路径（HKCU 下，无需管理员权限）
fn registry_key() -> String {
    format!(r"Software\Classes\{}", PROTOCOL)
}

/// 读取注册表登记的 exe 路径
///
/// 从 `HKCU\Software\Classes\molaunch\shell\open\command` 的默认值
/// （形如 `"C:\path\MoLaunch.exe" "%1"`）中解析出引号内的 exe 路径。
pub(super) fn registered_exe_windows() -> Option<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(format!(r"{}\shell\open\command", registry_key()))
        .ok()?;
    let cmd: String = key.get_value("").ok()?;
    // 解析引号包裹的 exe 路径（`"C:\path\MoLaunch.exe" "%1"`）
    let trimmed = cmd.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.split('"').next())
        .map(|s| s.to_string())
        .or_else(|| {
            // 无引号时取第一个空格前
            Some(trimmed.split_whitespace().next()?.to_string())
        })
}

/// Windows 注册 molaunch:// 协议（写 HKCU，免管理员）
pub(super) fn register_windows() -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let exe = super::current_exe_path().ok_or("无法获取当前 exe 路径")?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // HKCU\Software\Classes\molaunch
    let base = hkcu
        .create_subkey(registry_key())
        .map_err(|e| format!("创建协议键失败: {}", e))?
        .0;
    base.set_value("", &format!("URL:{} protocol", PROTOCOL))
        .map_err(|e| format!("写入协议描述失败: {}", e))?;
    base.set_value("URL Protocol", &"")
        .map_err(|e| format!("写入 URL Protocol 失败: {}", e))?;

    // DefaultIcon
    let icon_key = hkcu
        .create_subkey(format!(r"{}\DefaultIcon", registry_key()))
        .map_err(|e| format!("创建图标键失败: {}", e))?
        .0;
    icon_key
        .set_value("", &format!("{},0", exe))
        .map_err(|e| format!("写入图标路径失败: {}", e))?;

    // shell\open\command
    let cmd_key = hkcu
        .create_subkey(format!(r"{}\shell\open\command", registry_key()))
        .map_err(|e| format!("创建 command 键失败: {}", e))?
        .0;
    cmd_key
        .set_value("", &format!("\"{}\" \"%1\"", exe))
        .map_err(|e| format!("写入打开命令失败: {}", e))?;

    Ok(())
}

/// Windows 卸载 molaunch:// 协议（删除整个 HKCU 键）
pub(super) fn unregister_windows() -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.delete_subkey_all(registry_key()) {
        Ok(()) => Ok(()),
        Err(e) => {
            // 键不存在也算卸载成功（幂等）
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(format!("删除协议键失败: {}", e))
            }
        }
    }
}
