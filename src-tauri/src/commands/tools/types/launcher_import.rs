//! 启动器数据导入类型定义

use serde::{Deserialize, Serialize};

/// 支持的启动器来源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherKind {
    Pcl2,
    Pcl2Ce,
    Hmcl,
    MultiMc,
    Prism,
    Curseforge,
    Generic,
}

impl LauncherKind {
    /// UI 展示名
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pcl2 => "PCL2",
            Self::Pcl2Ce => "PCL2CE",
            Self::Hmcl => "HMCL",
            Self::MultiMc => "MultiMC",
            Self::Prism => "Prism Launcher",
            Self::Curseforge => "CurseForge",
            Self::Generic => "通用实例文件夹",
        }
    }
}

/// 可导入实例（探测阶段返回给前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportableInstance {
    /// 实例名
    pub name: String,
    /// 源实例路径
    pub path: String,
    /// 检测到的 Minecraft 版本（可能为空，需用户确认）
    pub mc_version: Option<String>,
    /// 检测到的加载器（forge/fabric/neoforge/optifine/liteloader/quilt/vanilla）
    pub loader: Option<String>,
    /// 加载器版本
    pub loader_version: Option<String>,
}

/// 一个启动器来源及其下的实例列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherSource {
    pub kind: LauncherKind,
    /// UI 展示名（如 "PCL2"）
    pub label: String,
    /// 启动器根路径（用于展示）
    pub base_path: String,
    pub instances: Vec<ImportableInstance>,
}

/// 单个实例导入请求
#[derive(Debug, Clone, Deserialize)]
pub struct LauncherImportRequest {
    /// 启动器类型
    pub kind: LauncherKind,
    /// 源实例路径
    pub source_path: String,
    /// 导入后的实例名（缺省取源实例目录名）
    pub instance_name: Option<String>,
    /// true=符号链接（不复制文件），false=复制
    pub symlink: bool,
}

/// 单个实例导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResultItem {
    /// 导入后的实例名
    pub name: String,
    pub success: bool,
    /// 成功描述或失败原因
    pub message: String,
    pub mc_version: Option<String>,
    pub loader: Option<String>,
}
