//! 内置深度链接 handler（`molaunch://run` / `install` / `open` 后缀路由）

use super::router::register_sync;
use super::security::validate_download_url;
use crate::log_error;
use crate::log_info;

/// 注册所有内置 handler（幂等，可重复调用）
pub fn register_builtin() {
    // molaunch://run?version=<version_id>
    //
    // 启动指定版本的游戏。当前实现为"登记 + 日志"，实际启动编排复用
    // version_launch_manager 的 launch action：此处仅示例化路由骨架，
    // 后续接入具体业务时在 handler 内调用对应 manager 即可。
    register_sync("run", |_app, req| {
        let version = req.get_str("version").unwrap_or("").to_string();
        log_info!("[Deeplink] run handler: version={}", version);
        if version.is_empty() {
            return Err("run handler 缺少 version 参数".into());
        }
        Ok(())
    });

    // molaunch://install?url=<pack_url>
    //
    // 安装整合包。**安全红线**：url 必须通过 security::validate_download_url
    // 的域名白名单校验（仅 https + media.forgecdn.net / modrinth / moiu.cn 等），
    // 防止恶意网站通过 molaunch://install 诱导启动器下载病毒。
    register_sync("install", |_app, req| {
        let Some(url) = req.get_str("url") else {
            return Err("install handler 缺少 url 参数".into());
        };
        match validate_download_url(url) {
            Ok(()) => {
                log_info!(
                    "[Deeplink] install handler: url={}（已通过白名单校验）",
                    url
                );
                Ok(())
            }
            Err(reason) => {
                log_error!("[Deeplink] install handler 拦截非法 url: {}", reason);
                Err(format!("install 下载地址不合法: {}", reason))
            }
        }
    });

    // molaunch://open?page=<page_path>
    //
    // 打开前端指定页面。前端监听 deeplink://new 事件后自行路由，
    // 这里仅做日志登记（保证有对应 handler 存在）。
    register_sync("open", |_app, req| {
        let page = req.get_str("page").unwrap_or("").to_string();
        log_info!("[Deeplink] open handler: page={}", page);
        Ok(())
    });
}
