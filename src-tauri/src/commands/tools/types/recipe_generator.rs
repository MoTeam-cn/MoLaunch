use serde::{Deserialize, Serialize};

/// 数据包导出请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeGeneratorExportParams {
    pub path: String,
    pub files: Vec<RecipeGeneratorPackFile>,
}

/// 数据包内单个文件（zip 内路径 + UTF-8 文本内容）
#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeGeneratorPackFile {
    pub path: String,
    pub content: String,
}
