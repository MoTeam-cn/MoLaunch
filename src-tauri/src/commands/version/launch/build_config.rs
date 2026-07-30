//! 启动配置构建
//!
//! 从全局配置 + 版本独立设置 + 前端入参解析出完整的 LaunchConfig

use crate::minecraft::launch::{AuthInfo, LaunchConfig};
use crate::minecraft::version::setup::VersionSetup;
use crate::state::{resolve_game_dir, AppState};
use crate::{log_info, log_warn};

use super::{parse_server_enter, resolve_game_language};

/// 构建启动配置
///
/// 从全局配置 + 版本独立设置 + 前端入参解析出完整的 LaunchConfig。
/// 包含：Java 路径、服务器地址、额外参数、内存、认证信息、离线皮肤、隔离模式等。
///
/// 安全修复：从后端 auth_storage 获取 access_token，避免前端 IPC 明文传输 token。
/// 前端只传 username 和 uuid，后端根据 uuid 从注册表加载对应账号的 token。
pub(super) async fn build_launch_config(
    state: &AppState,
    app_handle: &tauri::AppHandle,
    version_id: &str,
    java_path: Option<String>,
    username: String,
    uuid: String,
    login_type: Option<String>,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<String>,
    server_port: Option<u32>,
    extra_jvm_args_override: Option<Vec<String>>,
) -> LaunchConfig {
    let config = state.config.lock().await;
    let game_dir = resolve_game_dir(&config.game_dir);

    // 读取版本独立设置（setup.ini）
    let version_dir = game_dir.join("versions").join(version_id);
    let setup = VersionSetup::load_or_create(&version_dir, version_id);

    // Java 路径解析（根据 setup.java.java_mode 决定策略）：
    // - 前端传入的 java_path 优先级最高（兼容旧调用方）
    // - 否则按版本独立设置的 JavaMode 处理：
    //   - auto/空 → 自动选择（resolved_java = None，pipeline 按规则表选）
    //   - auto_version → 自动选择指定版本范围（pipeline 用 java_version_min/max 约束）
    //   - folder → 使用版本文件夹下的 Java（pipeline 查找 version_dir/runtime/）
    //   - custom → 使用 setup.java.java_path
    let resolved_java = java_path.or_else(|| {
        let mode = setup.java.java_mode.as_deref().unwrap_or("").trim();
        if mode.eq_ignore_ascii_case("custom") {
            setup.java.java_path.clone().filter(|s| !s.is_empty())
        } else {
            None
        }
    });
    let resolved_java_mode = setup.java.java_mode.clone();
    let resolved_java_version_min = setup.java.java_version_min.unwrap_or(0);
    let resolved_java_version_max = setup.java.java_version_max.unwrap_or(0);

    // 服务器：前端未传则用版本独立的 server_enter（"IP:Port" 格式需解析）
    let (resolved_server_addr, resolved_server_port) =
        if server_address.is_some() || server_port.is_some() {
            (server_address, server_port)
        } else if let Some(ref enter) = setup.display.server_enter {
            if !enter.is_empty() {
                parse_server_enter(enter)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

    // 额外参数：按空白拆分
    let split_args = |s: &Option<String>| -> Vec<String> {
        s.as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    };
    let mut extra_jvm_args = split_args(&setup.advanced.jvm_args);
    // 联机模块临时追加的 JVM 参数（单次启动有效，不持久化到 setup.ini）
    // 用途：联机启动 MC 时追加 -Djava.net.preferIPv4Stack=true，确保虚拟局域网通信正常
    if let Some(override_args) = extra_jvm_args_override {
        extra_jvm_args.extend(override_args);
    }
    let extra_game_args = split_args(&setup.advanced.game_args);
    let pre_launch_cmd = setup.advanced.run_cmd.clone().filter(|s| !s.is_empty());

    // 内存：版本独立设置 > 全局
    // - setup.java.memory_mode = Some("auto") → 根据系统内存动态计算（版本独立自动）
    // - setup.java.memory_mode = Some("custom") → 使用 setup.java.min_memory/max_memory
    // - None / 空 / 其他 → 回退到全局 config.memory.min/max_memory
    let (resolved_min_mem, resolved_max_mem) = match setup.java.memory_mode.as_deref().filter(|s| !s.is_empty()) {
        Some("auto") => crate::minecraft::system::suggest_memory(),
        Some("custom") => {
            let max = setup.java.max_memory.unwrap_or(config.memory.max);
            let min = setup.java.min_memory.unwrap_or_else(|| max / 2);
            (min, max)
        }
        _ => (config.memory.min, config.memory.max),
    };

    // 构建认证信息
    let login_type_str = login_type.unwrap_or_else(|| "Legacy".to_string());
    let is_legacy = login_type_str == "Legacy";
    let (access_token, client_token, server_url) = {
        match state.auth_storage.load().await {
            Ok(auth_state) => {
                if let Some(ref current) = auth_state.current_user {
                    // 验证 current_user 的 uuid 与前端传入的 uuid 一致（防止越权）
                    if current.uuid == uuid {
                        (
                            current.access_token.clone(),
                            current.client_token.clone(),
                            current.server_url.clone(),
                        )
                    } else {
                        log_warn!(
                            "当前登录账号 UUID ({}) 与请求的 UUID ({}) 不一致，使用空 token",
                            current.uuid,
                            uuid
                        );
                        (String::new(), String::new(), None)
                    }
                } else {
                    // 未登录或离线模式，token 为空
                    (String::new(), String::new(), None)
                }
            }
            Err(e) => {
                log_warn!("从 auth_storage 加载 token 失败: {}，使用空 token", e);
                (String::new(), String::new(), None)
            }
        }
    };

    let auth_info = AuthInfo {
        username,
        uuid,
        access_token,
        client_token,
        login_type: login_type_str,
        server_url,
    };

    // 离线账号皮肤：根据用户选择的皮肤变体调整 UUID
    // 方案 A：通过递增 UUID 末位让 MC 离线模式哈希到目标皮肤模型（Steve/Alex）
    let auth_info = if is_legacy {
        match state.auth_storage.load().await {
            Ok(auth_state) => {
                if let Some(acc) = auth_state
                    .offline_accounts
                    .iter()
                    .find(|a| a.uuid == auth_info.uuid)
                {
                    if let Some(ref skin_name) = acc.skin {
                        // 判断目标皮肤变体：slim → true（Alex 模型），classic → false（Steve 模型）
                        // 自定义皮肤格式 custom:/path|slim 或 custom:/path|classic
                        let slim = if skin_name.starts_with("custom:") {
                            skin_name.contains("|slim")
                        } else {
                            matches!(
                                skin_name.as_str(),
                                "Alex" | "Ari" | "Efe" | "Makena" | "Noor" | "Sunny" | "Zuri"
                            )
                        };
                        let adjusted_uuid =
                            crate::minecraft::auth::adjust_uuid_for_skin_variant(&auth_info.uuid, slim);
                        if adjusted_uuid != auth_info.uuid {
                            log_info!(
                                "离线皮肤 UUID 调整: {} -> {} (skin={}, slim={})",
                                auth_info.uuid,
                                adjusted_uuid,
                                skin_name,
                                slim
                            );
                        }
                        AuthInfo {
                            uuid: adjusted_uuid,
                            ..auth_info
                        }
                    } else {
                        auth_info
                    }
                } else {
                    auth_info
                }
            }
            Err(e) => {
                log_warn!("加载离线账号皮肤失败: {}, 使用原始 UUID", e);
                auth_info
            }
        }
    } else {
        auth_info
    };

    // 方案 B：离线账号皮肤资源包替换
    // 生成资源包 zip 替换原版玩家纹理，确保 1.19.3+ 也精确显示选定角色
    if is_legacy {
        let skin_to_apply = state.auth_storage.load().await.ok().and_then(|s| {
            s.offline_accounts
                .iter()
                .find(|a| a.uuid == auth_info.uuid)
                .and_then(|a| a.skin.clone())
        });

        match crate::minecraft::launch::skin_resourcepack::apply_skin_resourcepack(
            &game_dir,
            version_id,
            skin_to_apply.as_deref(),
        ) {
            Ok(_) => {}
            Err(e) => log_warn!("离线皮肤资源包生成失败: {}", e),
        }
    } else {
        // 非离线账号：清理可能存在的离线皮肤资源包
        crate::minecraft::launch::skin_resourcepack::remove_skin_resourcepack(&game_dir);
    }

    // 创建启动配置
    LaunchConfig {
        game_dir: game_dir.clone(),
        version_id: version_id.to_string(),
        auth_info: auth_info.clone(),
        min_memory: resolved_min_mem,
        max_memory: resolved_max_mem,
        window_width,
        window_height,
        server_address: resolved_server_addr,
        server_port: resolved_server_port,
        // 版本独立隔离设置覆盖全局
        isolation_mode: super::super::list::resolve_isolation_mode(
            &game_dir,
            version_id,
            config.isolation_mode,
        ),
        java_path: resolved_java,
        java_mode: resolved_java_mode,
        java_version_min: resolved_java_version_min,
        java_version_max: resolved_java_version_max,
        download_source: config.download.source.clone(),
        mirror_url: config.download.mirror_url.clone(),
        // 启动时文件补全用：从全局 config 读取下载参数（替代之前 validate.rs 硬编码 8/4/0）
        max_threads: config.download.max_threads,
        chunk_count: config.download.chunk_count,
        speed_limit: config.download.max_speed,
        extra_jvm_args,
        extra_game_args,
        pre_launch_cmd,
        // 启动高级选项：版本独立覆盖全局（两者都未禁用才启用）
        disable_jlw: config.launch_advanced.disable_jlw || setup.advanced.disable_jlw.unwrap_or(false),
        disable_lua: config.launch_advanced.disable_lua || setup.advanced.disable_lua.unwrap_or(false),
        // 忽略 Java 兼容性警告（仅版本独立设置，custom 模式下跳过兼容性校验）
        ignore_java_warning: setup.advanced.ignore_java_warning.unwrap_or(false),
        // 关闭文件校验（仅版本独立设置，跳过 libraries/assets/主 jar 文件校验和补全）
        disable_assets_verify: setup.advanced.disable_assets_verify.unwrap_or(false),
        // 使用高性能显卡（仅全局设置，启动前写注册表 GpuPreference=2）
        use_dedicated_gpu: config.launch_advanced.use_dedicated_gpu,
        // 自定义信息（→ ${version_type} 替换）
        custom_info: setup.display.custom_info.clone(),
        // 自定义窗口标题（→ Win32 SetWindowText）
        window_title: setup.display.window_title.clone(),
        // 游戏默认语言：none 模式不设置，auto 旧配置兼容跟随启动器语言
        game_language: resolve_game_language(&config.game_language, &config.language),
        app_handle: Some(app_handle.clone()),
    }
}
