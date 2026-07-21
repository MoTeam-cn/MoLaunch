//! Game launch module
//!
//! This module provides Minecraft launch functionality:
//! - Build launch arguments (JVM args, game args, classpath)
//! - Launch game process
//! - Version isolation support
//! - Complete launch pipeline
//! - Game process monitoring and crash detection
//!
//! Architecture:
//! - pipeline.rs:  完整的启动流水线，支持并行执行和进度追踪
//! - watcher.rs:   游戏进程监控和崩溃检测
//! - mod.rs:       模块入口与公共类型（LaunchArguments / AuthInfo）
//! - arguments.rs: 启动参数构建编排入口（build_launch_arguments）
//! - classpath.rs: Classpath 构建（含继承版本递归）
//! - jvm_args.rs:  JVM 参数构建（含 LUA / JLW 拆分）
//! - game_args.rs: 游戏参数构建
//! - embedded.rs:  嵌入资源释放与库检测

use serde::{Deserialize, Serialize};

pub mod pipeline;
pub mod watcher;

mod arguments;
pub(crate) mod classpath;
mod embedded;
mod game_args;
mod jvm_args;
pub mod skin_resourcepack;

// Re-export pipeline types
pub use pipeline::{
    LaunchConfig, LaunchPipeline, LaunchProgress, LaunchResult as PipelineLaunchResult, LaunchStage,
};

// Re-export watcher types
pub use watcher::{CrashCategory, CrashInfo, ExitInfo, GameState, GameWatcher, LoadProgress};

// Re-export main entry
pub use arguments::build_launch_arguments;

/// Launch arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchArguments {
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub main_class: String,
    pub classpath: String,
    pub version_id: String,
    pub game_dir: String,
    pub assets_dir: String,
    pub asset_index: String,
    pub auth_info: AuthInfo,
}

/// Auth info
///
/// 注意：手动实现 Debug，access_token 和 client_token 脱敏为 "***"，
/// 避免误用 {:?} 打印时泄露 token 到日志文件
#[derive(Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub client_token: String,
    pub login_type: String,
}

impl std::fmt::Debug for AuthInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthInfo")
            .field("username", &self.username)
            .field("uuid", &self.uuid)
            .field("access_token", &"***")
            .field("client_token", &"***")
            .field("login_type", &self.login_type)
            .finish()
    }
}

/// 敏感参数名列表（小写匹配）
const SENSITIVE_ARG_NAMES: &[&str] = &[
    "--accesstoken",
    "--uuid",
    "--session",
    "--clienttoken",
    "--password",
    "--refreshtoken",
];

/// 对启动参数列表进行脱敏，用于日志打印
///
/// 识别 `--accessToken <token>` 这类敏感参数，将其值替换为 `***`。
/// 参数名本身保留（方便调试），仅值脱敏。
pub(crate) fn sanitize_args_for_log(args: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(args.len());
    let mut skip_next = false;

    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            // 上一个参数是敏感参数，这个是它的值，脱敏
            result.push("***".to_string());
            skip_next = false;
            continue;
        }

        let lower = arg.to_lowercase();
        if SENSITIVE_ARG_NAMES.contains(&lower.as_str()) {
            // 敏感参数名，保留，但下一个参数（值）需要脱敏
            result.push(arg.clone());
            // 检查下一个参数是否存在且不是另一个参数（不以 -- 开头）
            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                skip_next = true;
            }
        } else {
            result.push(arg.clone());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_args() {
        let args = vec![
            "--username".to_string(),
            "player".to_string(),
            "--accessToken".to_string(),
            "eyJhbGciOiJIUzI1NiJ9.secret.token".to_string(),
            "--uuid".to_string(),
            "abc-123".to_string(),
            "--version".to_string(),
            "1.16.5".to_string(),
        ];
        let sanitized = sanitize_args_for_log(&args);
        assert_eq!(sanitized[1], "player");
        assert_eq!(sanitized[3], "***"); // accessToken 值脱敏
        assert_eq!(sanitized[5], "***"); // uuid 值脱敏
        assert_eq!(sanitized[7], "1.16.5"); // 普通参数不脱敏
    }

    #[test]
    fn test_auth_info_debug() {
        let auth = AuthInfo {
            username: "test".to_string(),
            uuid: "uuid".to_string(),
            access_token: "secret_token".to_string(),
            client_token: "client_secret".to_string(),
            login_type: "Microsoft".to_string(),
        };
        let debug_str = format!("{:?}", auth);
        assert!(debug_str.contains("***"));
        assert!(!debug_str.contains("secret_token"));
        assert!(!debug_str.contains("client_secret"));
    }
}
