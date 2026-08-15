use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 资源包编辑器 - 打开请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct RpOpenParams {
    /// zip 或文件夹路径
    pub path: String,
    /// 上一个 zip 会话的临时工作目录（打开新包时清理；folder 会话传空）
    #[serde(default)]
    pub previous_work_dir: Option<String>,
}

/// 资源包编辑器 - 打开结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RpOpenResult {
    /// 工作目录（zip 会话为临时目录，folder 会话为原目录；读取/保存均基于此）
    pub work_dir: String,
    /// 是否为 zip 包
    pub is_zip: bool,
    /// 包名（文件名或目录名）
    pub name: String,
    /// 格式：zip / folder
    pub format: String,
    /// 大小（字节，zip 为压缩包大小，folder 为递归总大小）
    pub size: u64,
    /// 包图标（pack.png base64 data URI），无则为 None
    pub icon_data_url: Option<String>,
    /// 原包路径（zip 会话为源 zip 路径，folder 会话为原目录路径；保存回原包时使用）
    pub src_path: Option<String>,
    /// pack.mcmeta 的 pack_format，缺失/解析失败时为 None
    pub pack_format: Option<u32>,
    /// pack_format 对应的 MC 版本范围描述（如 "1.20.5–1.21.x"）
    pub mc_version: Option<String>,
    /// pack.mcmeta 的 description，缺失/解析失败时为 None
    pub description: Option<String>,
    /// 结构树（根节点即包根目录）
    pub tree: RpTreeNode,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// 资源包结构树节点
#[derive(Debug, Serialize, Deserialize)]
pub struct RpTreeNode {
    /// 名称
    pub name: String,
    /// 相对包根的路径（正斜杠分隔）
    pub rel_path: String,
    /// dir / file
    pub kind: String,
    /// 文件类型：mcmeta / png / model / lang / json / ogg / text / other
    pub file_type: String,
    /// 大小（字节，目录为子树合计）
    pub size: u64,
    /// 是否为动画纹理（同目录存在同名 .png.mcmeta）
    pub animated: bool,
    /// 子节点（文件为空）
    pub children: Vec<RpTreeNode>,
}

/// 资源包编辑器 - 读取单文件请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct RpReadParams {
    /// 工作目录（rp_open 返回）
    pub work_dir: String,
    /// 包内相对路径（正斜杠）
    pub rel_path: String,
}

/// 资源包编辑器 - 读取单文件结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RpReadResult {
    /// 内容类型：text（原文）/ data_uri（base64 data URI）
    pub kind: String,
    /// 文件内容
    pub content: String,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// 资源包编辑器 - 批量读取模型与关联纹理请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct RpReadManyParams {
    /// 工作目录（rp_open 返回）
    pub work_dir: String,
    /// 起始文件：blockstates/*.json 或 models/**/*.json（相对包根）
    pub rel_path: String,
}

/// 资源包编辑器 - 批量读取结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RpReadManyResult {
    /// 起始文件相对路径
    pub root: String,
    /// rel_path -> 文件内容（model/blockstate 为 text，关联纹理为 data_uri）
    pub files: BTreeMap<String, RpReadResult>,
    /// 失败原因（成功时为空）
    pub error: String,
}

/// 资源包编辑器 - 写回单文件请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct RpWriteParams {
    /// 工作目录（rp_open 返回）
    pub work_dir: String,
    /// 包内相对路径（正斜杠）
    pub rel_path: String,
    /// 内容类型：text（UTF-8 文本）/ base64（图片等二进制，允许带 data URI 前缀）
    pub kind: String,
    /// 内容（text 为原文；base64 为编码数据）
    pub content: String,
}

/// 资源包编辑器 - 写回结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RpWriteResult {
    /// 是否成功
    pub success: bool,
    /// 提示信息
    pub message: String,
}

/// 资源包编辑器 - 导出请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct RpExportParams {
    /// 工作目录（rp_open 返回）
    pub work_dir: String,
    /// 导出目标路径：保存回原包时传原 zip 路径，另存为时传用户选择路径
    pub path: String,
    /// 导出格式：zip（当前仅支持 zip 打包）
    pub format: String,
    /// 原 zip 路径（zip 会话的源包，另存时复制其注释等附加属性），无则 None
    #[serde(default)]
    pub src_path: Option<String>,
}

/// 资源包编辑器 - 导出结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RpExportResult {
    /// 是否成功
    pub success: bool,
    /// 输出路径
    pub output_path: String,
    /// 提示信息
    pub message: String,
}
