//! 应用状态管理

use crate::sdk::SdkInstance;
use crate::{log_info, log_warn};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

/// 本地认证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAuthResult {
    /// 用户名
    pub name: String,
    /// UUID
    pub uuid: String,
    /// 访问令牌
    pub access_token: String,
    /// 客户端令牌
    pub client_token: String,
    /// 登录类型
    pub login_type: String,
    /// 微软登录时的档案信息
    pub profile_json: Option<String>,
}

/// 启动历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchHistory {
    /// 版本ID
    pub version_id: String,
    /// 用户名
    pub username: String,
    /// 启动时间
    pub launch_time: String,
    /// 进程ID
    pub pid: u32,
    /// 退出码（如果有）
    pub exit_code: Option<i32>,
}

/// 应用全局状态
pub struct AppState {
    pub sdk: Arc<TokioMutex<Option<SdkInstance>>>,
    pub config: Arc<TokioMutex<AppConfig>>,
    pub auth: Arc<TokioMutex<AuthState>>,
    pub auth_storage: Arc<crate::minecraft::auth::storage::AuthStorage>,
    pub download_state: Arc<Mutex<DownloadState>>,
    pub launch_history: Arc<TokioMutex<Vec<LaunchHistory>>>,
    pub current_pid: Arc<TokioMutex<Option<u32>>>,
    pub launch_pipeline: Arc<TokioMutex<Option<Arc<crate::minecraft::launch::LaunchPipeline>>>>,
}

/// 阶段状态
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum StageStatus {
    #[default]
    Waiting,
    Loading,
    Finished,
    Failed,
}

/// 下载阶段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStage {
    pub name: String,
    pub progress: f64,
    pub weight: f64,
    pub status: StageStatus,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub files_downloaded: usize,
    pub files_total: usize,
    /// 所属任务分组（用于前端按"整合包安装"/"MC本体安装"等分组折叠展开）
    /// None 表示独立阶段（不分组），Some 表示归属于某分组
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl DownloadStage {
    pub fn new(name: impl Into<String>, weight: f64) -> Self {
        Self {
            name: name.into(),
            progress: 0.0,
            weight,
            status: StageStatus::Waiting,
            bytes_downloaded: 0,
            bytes_total: 0,
            files_downloaded: 0,
            files_total: 0,
            group: None,
        }
    }

    /// 创建带分组的 stage
    pub fn new_grouped(name: impl Into<String>, weight: f64, group: impl Into<String>) -> Self {
        let mut s = Self::new(name, weight);
        s.group = Some(group.into());
        s
    }
}

/// 下载状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadState {
    pub is_active: bool,
    pub is_complete: bool,
    pub stages: Vec<DownloadStage>,
    pub current_stage_index: usize,
    pub global_speed: u64,
    pub global_bytes_downloaded: u64,
    pub global_bytes_total: u64,
    pub error_code: i32,
}

impl Default for DownloadState {
    fn default() -> Self {
        Self {
            is_active: false,
            is_complete: false,
            stages: vec![
                DownloadStage::new("版本清单", 2.0),
                DownloadStage::new("版本信息", 3.0),
                DownloadStage::new("客户端", 5.0),
                DownloadStage::new("库文件", 15.0),
                DownloadStage::new("资源文件", 20.0),
            ],
            current_stage_index: 0,
            global_speed: 0,
            global_bytes_downloaded: 0,
            global_bytes_total: 0,
            error_code: 0,
        }
    }
}

impl DownloadState {
    /// 重置为指定 stages（清空原有，用于独立安装流程）
    pub fn reset_stages(&mut self, stages: Vec<DownloadStage>) {
        self.stages = stages;
        self.current_stage_index = 0;
        self.is_active = true;
        self.is_complete = false;
        self.global_speed = 0;
        self.global_bytes_downloaded = 0;
        self.global_bytes_total = 0;
        self.error_code = 0;
    }

    /// 追加 stages（保留原有，用于连续安装流程：整合包 → MC 本体）
    /// 返回追加前的 stages 长度，作为后续 stage_callback 的偏移量
    pub fn append_stages(&mut self, stages: Vec<DownloadStage>) -> usize {
        let offset = self.stages.len();
        self.stages.extend(stages);
        self.is_active = true;
        self.is_complete = false;
        offset
    }

    /// 设置当前阶段索引（stage_callback 调用）
    /// 自动把前一阶段标记为 Finished（仅当 idx > prev 时）
    pub fn set_current_stage(&mut self, idx: usize) {
        if idx > self.current_stage_index && self.current_stage_index < self.stages.len() {
            self.stages[self.current_stage_index].status = StageStatus::Finished;
            self.stages[self.current_stage_index].progress = 1.0;
        }
        self.current_stage_index = idx;
        if idx < self.stages.len() {
            self.stages[idx].status = StageStatus::Loading;
            self.stages[idx].progress = 0.0;
            self.stages[idx].bytes_downloaded = 0;
            self.stages[idx].bytes_total = 0;
        }
    }

    /// 设置指定阶段的状态和进度（本地操作用：解析 zip、复制 overrides 等）
    pub fn set_stage_status(&mut self, idx: usize, status: StageStatus, progress: f64) {
        self.current_stage_index = idx;
        if idx < self.stages.len() {
            self.stages[idx].status = status;
            self.stages[idx].progress = progress;
        }
    }

    /// 设置指定阶段的字节进度（本地操作如解压 overrides）
    pub fn set_stage_bytes(&mut self, idx: usize, downloaded: u64, total: u64) {
        if idx < self.stages.len() {
            self.stages[idx].bytes_downloaded = downloaded;
            self.stages[idx].bytes_total = total;
            if total > 0 {
                self.stages[idx].progress = (downloaded as f64 / total as f64).min(1.0);
            }
        }
    }

    /// 同步 DownloadManager 的 GlobalProgress 到指定阶段 + 更新全局指标
    /// 这是核心统一方法：整合包/MC 本体/自定义下载都用这个
    /// 统一规则：
    ///   - stage 进度按 bytes 计算（total_bytes>0 时），否则按 files 计算
    ///   - global_bytes 累加所有 Finished + Loading 阶段（支持连续安装流程的进度连贯）
    ///   - global_speed 直接信任 DownloadManager 的 current_speed（它已有 300ms 滑动窗口）
    pub fn sync_stage_from_progress(
        &mut self,
        idx: usize,
        downloaded_bytes: u64,
        total_bytes: u64,
        completed_files: usize,
        total_files: usize,
        current_speed: u64,
    ) {
        if idx < self.stages.len() {
            let stage = &mut self.stages[idx];
            stage.bytes_downloaded = downloaded_bytes;
            stage.bytes_total = total_bytes;
            stage.files_downloaded = completed_files;
            stage.files_total = total_files;
            stage.status = StageStatus::Loading;
            if total_bytes > 0 {
                stage.progress = (downloaded_bytes as f64 / total_bytes as f64).min(1.0);
            } else if total_files > 0 && completed_files >= total_files {
                stage.progress = 1.0;
            }
        }

        // 统一 global_bytes 算法：累加所有 Finished + Loading 阶段
        let mut g_downloaded = 0u64;
        let mut g_total = 0u64;
        for stage in &self.stages {
            if stage.status == StageStatus::Finished || stage.status == StageStatus::Loading {
                g_downloaded += stage.bytes_downloaded;
                g_total += stage.bytes_total;
            }
        }
        self.global_bytes_downloaded = g_downloaded;
        self.global_bytes_total = g_total;
        self.global_speed = current_speed;
    }

    /// 标记整体完成（所有 Loading 阶段标记为 Finished）
    pub fn mark_complete(&mut self) {
        self.is_active = false;
        self.is_complete = true;
        for stage in &mut self.stages {
            if stage.status == StageStatus::Loading {
                stage.status = StageStatus::Finished;
                stage.progress = 1.0;
            }
        }
    }

    /// 标记整体失败
    pub fn mark_failed(&mut self, error_code: i32) {
        self.is_active = false;
        self.is_complete = false;
        self.error_code = error_code;
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub game_dir: String,
    /// Minecraft 文件夹列表（含默认和用户添加的）
    pub mc_folders: Vec<McFolder>,
    pub max_download_threads: u32,
    pub chunk_count: u32,
    pub isolation_mode: u32,
    pub log_level: u32,
    pub min_memory: u32,
    pub max_memory: u32,
    pub memory_mode: String, // "auto" | "custom"
    pub theme: String,
    pub language: String,
    pub mirror_url: Option<String>,
    pub mirror_url_meta: Option<String>,
    pub mirror_url_download: Option<String>,
    pub mirror_mode: u32,
    pub max_download_speed: u64,
    pub download_source: String, // "mirror" | "official" | "smart" — 文件下载源
    pub meta_source: String,    // "mirror" | "official" | "smart" — 版本列表源
    pub proxy_mode: String,      // "none" | "system" | "custom"
    pub proxy_type: String,      // "http" | "https" | "socks5"
    pub proxy_url: String,       // 自定义代理地址，如 "127.0.0.1:7890"
    /// 上次选中的游戏版本（持久化，启动器重启后恢复）
    pub selected_version: Option<String>,
    // ===== 社区资源配置（参考 PCL2 PageSetupSystem "社区资源" 卡片）=====
    /// 社区资源来源策略：0=尽量镜像 / 1=缓慢时换镜像 / 2=尽量官方（默认 2）
    pub community_source: u8,
    /// 下载文件名格式：0=【译名】原名 / 1=[译名] 原名 / 2=译名-原名 / 3=原名-译名 / 4=仅原名（默认 1）
    pub community_filename_format: u8,
    /// Mod 管理页显示样式：0=标题译名/详情文件名 / 1=标题文件名/详情译名（默认 0）
    pub community_mod_local_name_style: u8,
    /// 在显示 Mod 加载器时忽略 Quilt（默认 true）
    pub community_ignore_quilt: bool,
}

/// Minecraft 文件夹项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McFolder {
    /// 显示名称
    pub name: String,
    /// 文件夹路径（相对或绝对）
    pub path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_game_dir = get_default_game_dir();
        Self {
            game_dir: default_game_dir.clone(),
            mc_folders: vec![McFolder {
                name: "默认".to_string(),
                path: default_game_dir,
            }],
            max_download_threads: 8,
            chunk_count: 4,
            isolation_mode: 4,
            log_level: 3,
            min_memory: 0,
            max_memory: 0,
            memory_mode: "auto".to_string(),
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            mirror_url: None,
            mirror_url_meta: None,
            mirror_url_download: None,
            mirror_mode: 0,
            max_download_speed: 0,
            download_source: "smart".to_string(),
            meta_source: "smart".to_string(),
            proxy_mode: "none".to_string(),
            proxy_type: "http".to_string(),
            proxy_url: String::new(),
            selected_version: None,
            community_source: 2,
            community_filename_format: 1,
            community_mod_local_name_style: 0,
            community_ignore_quilt: true,
        }
    }
}

/// 认证状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthState {
    pub current_user: Option<LocalAuthResult>,
    pub is_logged_in: bool,
}

/// 获取默认游戏目录：启动器同级目录下的 .minecraft
fn get_default_game_dir() -> String {
    // 优先使用可执行文件所在目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let minecraft_dir = exe_dir.join(".minecraft");
            return minecraft_dir.to_string_lossy().to_string();
        }
    }
    // 兜底：当前工作目录
    if let Ok(cwd) = std::env::current_dir() {
        let minecraft_dir = cwd.join(".minecraft");
        return minecraft_dir.to_string_lossy().to_string();
    }
    ".minecraft".to_string()
}

/// 解析游戏目录：如果是相对路径，则相对于可执行文件目录
pub fn resolve_game_dir(game_dir: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(game_dir);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    // 相对路径：优先相对于可执行文件目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join(game_dir);
        }
    }
    // 兜底：当前工作目录
    std::env::current_dir().unwrap_or_default().join(game_dir)
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        // 尝试从文件加载配置
        let config = match crate::config::load_config() {
            Ok(Some(config)) => {
                log_info!("Loaded config from file");
                config
            }
            Ok(None) => {
                log_info!("No config file found, using defaults");
                AppConfig::default()
            }
            Err(e) => {
                log_warn!("Failed to load config: {}, using defaults", e);
                AppConfig::default()
            }
        };

        // 尝试加载 SDK lite
        let sdk = match crate::sdk::SdkInstance::load() {
            Ok(sdk) => {
                log_info!("SDK lite loaded successfully");
                Some(sdk)
            }
            Err(e) => {
                log_warn!("Failed to load SDK lite: {}", e);
                None
            }
        };

        // 创建 SDK Arc（需先创建以便共享给 auth_storage）
        let sdk_arc = Arc::new(TokioMutex::new(sdk));

        Self {
            sdk: sdk_arc.clone(),
            config: Arc::new(TokioMutex::new(config)),
            auth: Arc::new(TokioMutex::new(AuthState::default())),
            auth_storage: Arc::new(crate::minecraft::auth::storage::AuthStorage::new(sdk_arc)),
            download_state: Arc::new(Mutex::new(DownloadState::default())),
            launch_history: Arc::new(TokioMutex::new(Vec::new())),
            current_pid: Arc::new(TokioMutex::new(None)),
            launch_pipeline: Arc::new(TokioMutex::new(None)),
        }
    }
}
