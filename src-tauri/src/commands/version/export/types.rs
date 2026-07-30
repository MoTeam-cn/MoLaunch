//! 导出功能类型定义

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 导出整合包格式
///
/// 与导入支持的格式对齐（除 LauncherPack 和 Compress 兜底外均支持导出）。
/// - `Modrinth`：默认，生成 modrinth.index.json + overrides/
/// - `Curseforge`：生成 manifest.json（无 addons）+ overrides/ + modlist.html
/// - `Hmcl`：生成 modpack.json + minecraft/
/// - `Mmc`：生成 mmc-pack.json + instance.cfg + .minecraft/
/// - `Mcbbs`：生成 mcbbs.packmeta + overrides/
/// - `Compress`：直接打包 .minecraft/
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExportFormat {
    /// Modrinth 格式（.mrpack）
    Modrinth,
    /// CurseForge 格式（manifest.json + overrides/）
    Curseforge,
    /// HMCL 格式（modpack.json + minecraft/）
    Hmcl,
    /// MultiMC 格式（mmc-pack.json + .minecraft/）
    Mmc,
    /// MCBBS 格式（mcbbs.packmeta + overrides/）
    Mcbbs,
    /// 普通压缩包兜底（.minecraft/）
    Compress,
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Modrinth
    }
}

impl ExportFormat {
    /// 整合包文件扩展名（不含 `.`）
    pub fn extension(self) -> &'static str {
        match self {
            Self::Modrinth => "mrpack",
            _ => "zip",
        }
    }

    /// 是否需要联网检查 mod 下载地址
    ///
    /// 仅 Modrinth 和 CurseForge 格式有 mods 下载列表，其他格式直接打包文件。
    pub fn requires_online_check(self) -> bool {
        matches!(self, Self::Modrinth | Self::Curseforge)
    }
}

/// 单个导出选项
///
/// 一个选项包含标题、描述、文件匹配规则（`|` 分隔，`!` 开头表排除）。
/// 规则支持 `*`、`?`、`[abc]`、`[!abc]` 通配符，匹配相对路径（以 `/` 为分隔符）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOption {
    /// 选项唯一标识（如 "basic"、"mods"、"resourcepacks"）
    pub id: String,
    /// 显示标题（如 "Mod"、"资源包"）
    pub title: String,
    /// 描述（可空）
    pub description: Option<String>,
    /// 文件匹配规则（`|` 分隔，`!` 开头表排除）
    /// 如 "mods/|!mods/*.disabled|coremods/"
    pub rules: Option<String>,
    /// 仅用于判断是否显示（不参与导出），为空时用 rules 判断
    pub show_rules: Option<String>,
    /// 是否默认勾选
    pub default_checked: bool,
    /// 是否被勾选（由前端用户操作，导出时回传）
    #[serde(default)]
    pub checked: bool,
    /// 父选项 id（None=顶层，Some=子选项）
    pub parent: Option<String>,
    /// 是否可用（如 RequireModLoader 但版本无 mod 加载器时为 false）
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 是否可见（根据实际文件扫描结果决定）
    #[serde(default = "default_true")]
    pub visible: bool,
}

fn default_true() -> bool {
    true
}

/// 导出请求参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportModpackParams {
    /// 版本 ID（如 "1.20.1-Forge"）
    pub version_id: String,
    /// 整合包名称
    pub pack_name: String,
    /// 整合包版本号（如 "1.0.0"）
    pub pack_version: String,
    /// 用户勾选的导出选项（含 checked 状态）
    pub options: Vec<ExportOption>,
    /// 是否联网检查 mod 下载地址（true=联网，false=直接打包文件）
    #[serde(default = "default_true")]
    pub check_hosted_assets: bool,
    /// 仅从 Modrinth 查询（true=跳过 CurseForge）
    #[serde(default)]
    pub modrinth_upload_mode: bool,
    /// 导出文件保存路径（由前端文件对话框选择，或配置文件指定）
    pub config_pack_path: Option<String>,
    /// 导出格式（默认 Modrinth）
    #[serde(default)]
    pub format: ExportFormat,
}

/// 导出结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportModpackResult {
    pub success: bool,
    pub file_path: String,
    pub file_size: u64,
    /// 打包的文件总数
    pub file_count: usize,
    /// 联网获取到下载地址的 mod 数
    pub mod_count: usize,
}

/// 导出进度阶段
///
/// 与 `ExportProgress.stage` 字段对应，前端按阶段展示不同文案。
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExportStage {
    /// 初始化（解析参数、定位实例目录）
    Init,
    /// 扫描文件（应用规则）
    Scan,
    /// 联网检查 mod 下载地址（Modrinth + CurseForge）
    Network,
    /// 打包 zip（写入 manifest + overrides）
    Zip,
    /// 完成
    Done,
    /// 失败
    Failed,
}

/// 导出进度事件 payload
///
/// 通过 Tauri `emit("export-progress", payload)` 推送，前端 listen 后更新进度条。
/// `percent` 范围 0-100，`message` 为人类可读的当前操作描述。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgress {
    /// 当前阶段
    pub stage: ExportStage,
    /// 总进度百分比（0-100）
    pub percent: u8,
    /// 当前操作描述（如"扫描文件 234/567"）
    pub message: String,
    /// 版本 ID（用于前端区分是哪个版本的导出任务）
    pub version_id: String,
}

impl ExportProgress {
    pub fn new(stage: ExportStage, percent: u8, message: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            stage,
            percent: percent.min(100),
            message: message.into(),
            version_id: version_id.into(),
        }
    }
}

/// 扫描到的待导出文件信息
#[derive(Debug, Clone)]
pub struct ExportFileInfo {
    /// 相对于实例目录的路径（正斜杠分隔，如 "mods/xxx.jar"）
    pub relative_path: String,
    /// 绝对路径
    pub abs_path: PathBuf,
    /// 文件大小
    pub size: u64,
}

/// 联网检查获取到的 mod 下载信息
#[derive(Debug, Clone)]
pub struct ModDownloadInfo {
    /// 相对路径（对应 modrinth.index.json 中的 path）
    pub relative_path: String,
    /// sha1 hash
    pub sha1: String,
    /// sha512 hash
    pub sha512: String,
    /// 下载地址列表
    pub downloads: Vec<String>,
    /// 文件大小
    pub file_size: u64,
    /// CurseForge project id（仅 CF 查询结果设置，MR 为 None）
    /// 用于导出 CurseForge 格式整合包时写入 manifest.files[].projectID
    pub project_id: Option<i64>,
    /// CurseForge file id（仅 CF 查询结果设置，MR 为 None）
    /// 用于导出 CurseForge 格式整合包时写入 manifest.files[].fileID
    pub file_id: Option<i64>,
}

/// Modrinth 整合包索引文件（modrinth.index.json）
#[derive(Debug, Serialize)]
pub struct MrIndexJson {
    pub game: String,
    pub format_version: u32,
    pub version_id: String,
    pub name: String,
    pub summary: String,
    pub files: Vec<MrIndexFile>,
    pub dependencies: std::collections::HashMap<String, String>,
}

/// modrinth.index.json 中的单个文件条目
#[derive(Debug, Serialize)]
pub struct MrIndexFile {
    pub path: String,
    pub hashes: MrIndexHashes,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

/// modrinth.index.json 文件条目的 hashes 字段
#[derive(Debug, Serialize)]
pub struct MrIndexHashes {
    pub sha1: String,
    pub sha512: String,
}

/// 配置文件保存请求
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveConfigParams {
    pub config_path: String,
    pub pack_name: String,
    pub pack_version: String,
    pub check_hosted_assets: bool,
    pub modrinth_upload_mode: bool,
    pub pack_path: Option<String>,
    pub options: Vec<ExportOption>,
}

/// 配置文件读取结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadConfigResult {
    pub pack_name: String,
    pub pack_version: String,
    pub check_hosted_assets: bool,
    pub modrinth_upload_mode: bool,
    pub pack_path: Option<String>,
    /// 从配置文件读取的规则覆盖列表（直接作为导出规则使用）
    pub rules_override: Vec<String>,
}
