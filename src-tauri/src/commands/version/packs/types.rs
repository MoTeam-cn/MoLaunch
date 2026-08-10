//! Packs 数据类型（PackKind / PackInfo）

use serde::{Deserialize, Serialize};

/// 内容类型：资源包 / 光影
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PackKind {
    Resourcepack,
    Shader,
}

impl PackKind {
    /// 对应子目录名
    pub fn subdir(&self) -> &'static str {
        match self {
            PackKind::Resourcepack => "resourcepacks",
            PackKind::Shader => "shaderpacks",
        }
    }

    /// 允许的文件扩展名
    pub fn suffixes(&self) -> &'static [&'static str] {
        &["zip"]
    }
}

/// 单个资源包/光影信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackInfo {
    /// 文件名（含扩展名/目录名，可能带 .disabled / .old 后缀）
    pub file_name: String,
    /// 启用时的文件名（去除启停后缀）
    pub enabled_name: String,
    /// 是否启用
    pub is_enabled: bool,
    /// 是否为文件夹形态
    pub is_folder: bool,
    /// 大小（字节，文件夹为递归合计）
    pub size: u64,
}
