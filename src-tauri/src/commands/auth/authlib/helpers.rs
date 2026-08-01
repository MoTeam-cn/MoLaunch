//! authlib 命令共享的辅助函数（账号加载 / PNG 文件读取校验）

use crate::error_util::log_err;
use crate::minecraft::auth::storage::StoredAuthlibAccount;
use crate::state::AppState;

/// 从持久化存储读取 authlib 账号
///
/// 失败返回友好错误信息（前端可直接 toast 显示）。
/// 多处复用：5 个皮肤命令都需要先取账号再调用 client。
pub(super) async fn load_account_or_err(
    state: &AppState,
    server_url: &str,
    uuid: &str,
) -> Result<StoredAuthlibAccount, String> {
    state
        .auth_storage
        .get_authlib_account(server_url, uuid)
        .await
        .map_err(log_err("Failed to load authlib account"))?
        .ok_or_else(|| "authlib 账号不存在".to_string())
}

/// 读取 PNG 文件并校验文件头与大小
///
/// 与 `commands::auth::account::offline::save_custom_skin` 中的校验保持一致：
/// - PNG 文件头：`89 50 4E 47 0D`（5 字节）
/// - 大小限制：1MB（yggdrasil 服务端通常自行限制，这里仅做客户端预校验）
///
/// 在 `spawn_blocking` 中执行 `std::fs::read`，避免阻塞异步运行时
/// （与微软 `upload_skin` 命令的处理方式一致）。
pub(super) async fn read_png_file(file_path: &str) -> Result<Vec<u8>, String> {
    let path = file_path.to_string();
    tokio::task::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|e| format!("读取皮肤文件失败: {}", e))?;
        if bytes.len() < 8 || bytes[0..5] != [0x89, 0x50, 0x4E, 0x47, 0x0D] {
            return Err("文件不是有效的 PNG 格式".to_string());
        }
        if bytes.len() > 1024 * 1024 {
            return Err("皮肤文件过大（超过 1MB）".to_string());
        }
        Ok(bytes)
    })
    .await
    .map_err(|e| format!("读取文件任务失败: {}", e))?
}
