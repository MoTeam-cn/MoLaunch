//! 分类标签映射表
//!
//! 参考 PCL2 PageDownloadMod/Pack/ResourcePack/Shader/DataPack.xaml
//! 每个分类标签格式为 "CurseForgeId/ModrinthSlug"

use super::types::ResourceType;

/// 分类标签项
#[derive(Debug, Clone)]
pub struct CategoryTag {
    /// CurseForge 分类 ID
    pub curseforge_id: u32,
    /// Modrinth slug
    pub modrinth_slug: &'static str,
    /// 中文显示名
    pub label: &'static str,
}

impl CategoryTag {
    /// 从 "CFId/MrSlug" 格式字符串解析
    pub fn from_combined(combined: &'static str, label: &'static str) -> Self {
        let parts: Vec<&'static str> = combined.split('/').collect();
        let cf_id = parts
            .first()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let mr_slug = parts.get(1).copied().unwrap_or("");
        Self {
            curseforge_id: cf_id,
            modrinth_slug: mr_slug,
            label,
        }
    }

    /// 根据来源返回分类 ID 字符串
    pub fn id_for_source(&self, source: u32) -> String {
        if source == 1 {
            // 仅 CurseForge
            self.curseforge_id.to_string()
        } else if source == 2 {
            // 仅 Modrinth
            self.modrinth_slug.to_string()
        } else {
            // 全部：用 CurseForge ID
            self.curseforge_id.to_string()
        }
    }
}

/// 获取指定资源类型的分类标签列表
pub fn get_categories(rtype: ResourceType) -> Vec<CategoryTag> {
    match rtype {
        ResourceType::Mod => mod_categories(),
        ResourceType::ModPack => modpack_categories(),
        ResourceType::ResourcePack => resourcepack_categories(),
        ResourceType::Shader => shader_categories(),
        ResourceType::DataPack => datapack_categories(),
    }
}

/// 将 Modrinth 返回的分类 slug 翻译为中文（参考 PCL2 ResourceProject.vb:310-378）
pub fn translate_modrinth_tag(slug: &str) -> Option<&'static str> {
    // 先在 Mod 分类里找
    if let Some(t) = mod_categories().iter().find(|t| t.modrinth_slug == slug) {
        return Some(t.label);
    }
    // 再在整合包分类里找
    if let Some(t) = modpack_categories().iter().find(|t| t.modrinth_slug == slug) {
        return Some(t.label);
    }
    // 特殊处理：加载器标签不翻译
    match slug {
        "fabric" | "forge" | "neoforge" | "quilt" | "liteloader" => None,
        _ => None,
    }
}

/// 将 CurseForge 返回的分类 ID 翻译为中文（参考 PCL2 ResourceProject.vb:199-274）
pub fn translate_curseforge_tag(id: u32) -> Option<&'static str> {
    // 先在 Mod 分类里找
    if let Some(t) = mod_categories().iter().find(|t| t.curseforge_id == id) {
        return Some(t.label);
    }
    // 再在整合包分类里找
    if let Some(t) = modpack_categories().iter().find(|t| t.curseforge_id == id) {
        return Some(t.label);
    }
    None
}

/// Mod 分类（参考 PCL2 PageDownloadMod.xaml）
fn mod_categories() -> Vec<CategoryTag> {
    vec![
        CategoryTag::from_combined("406/worldgen", "世界元素"),
        CategoryTag::from_combined("407/", "生物群系"),
        CategoryTag::from_combined("410/", "维度"),
        CategoryTag::from_combined("408/", "矿物/资源"),
        CategoryTag::from_combined("409/", "天然结构"),
        CategoryTag::from_combined("412/technology", "科技"),
        CategoryTag::from_combined("415/", "管道/物流"),
        CategoryTag::from_combined("4843/", "自动化"),
        CategoryTag::from_combined("417/", "能源"),
        CategoryTag::from_combined("4558/", "红石"),
        CategoryTag::from_combined("436/", "食物/烹饪"),
        CategoryTag::from_combined("416/", "农业"),
        CategoryTag::from_combined("414/", "运输"),
        CategoryTag::from_combined("420/", "仓储"),
        CategoryTag::from_combined("419/magic", "魔法"),
        CategoryTag::from_combined("422/", "冒险"),
        CategoryTag::from_combined("424/", "装饰"),
        CategoryTag::from_combined("411/", "生物"),
        CategoryTag::from_combined("434/", "装备"),
        CategoryTag::from_combined("6814/optimization", "性能优化"),
        CategoryTag::from_combined("9026/", "创造模式"),
        CategoryTag::from_combined("423/", "信息显示"),
        CategoryTag::from_combined("435/", "服务器"),
        CategoryTag::from_combined("5191/", "改良"),
        CategoryTag::from_combined("421/library", "支持库"),
    ]
}

/// 整合包分类（参考 PCL2 PageDownloadPack.xaml）
fn modpack_categories() -> Vec<CategoryTag> {
    vec![
        CategoryTag::from_combined("4484/", "多人"),
        CategoryTag::from_combined("4479/challenging", "硬核"),
        CategoryTag::from_combined("4483/", "战斗"),
        CategoryTag::from_combined("4478/quests", "任务"),
        CategoryTag::from_combined("4472/technology", "科技"),
        CategoryTag::from_combined("4473/magic", "魔法"),
        CategoryTag::from_combined("4475/adventure", "冒险"),
        CategoryTag::from_combined("4476/", "探索"),
        CategoryTag::from_combined("4477/", "小游戏"),
        CategoryTag::from_combined("4474/scifi", "科幻"),
        CategoryTag::from_combined("4736/", "空岛"),
        CategoryTag::from_combined("5128/", "原版改良"),
        CategoryTag::from_combined("4487/", "FTB"),
        CategoryTag::from_combined("4480/", "基于地图"),
        CategoryTag::from_combined("4481/", "轻量"),
        CategoryTag::from_combined("4482/", "大型"),
    ]
}

/// 资源包分类（参考 PCL2 PageDownloadResourcePack.xaml）
fn resourcepack_categories() -> Vec<CategoryTag> {
    vec![
        CategoryTag::from_combined("403/", "原版风"),
        CategoryTag::from_combined("400/", "写实风"),
        CategoryTag::from_combined("401/", "现代风"),
        CategoryTag::from_combined("402/", "中世纪"),
        CategoryTag::from_combined("399/", "蒸汽朋克"),
        CategoryTag::from_combined("5244/", "含字体"),
        CategoryTag::from_combined("404/", "动态效果"),
        CategoryTag::from_combined("4465/", "兼容 Mod"),
        CategoryTag::from_combined("393/", "16x"),
        CategoryTag::from_combined("394/", "32x"),
        CategoryTag::from_combined("395/", "64x"),
        CategoryTag::from_combined("396/", "128x"),
        CategoryTag::from_combined("397/", "256x"),
        CategoryTag::from_combined("398/", "超高清"),
    ]
}

/// 光影分类（参考 PCL2 PageDownloadShader.xaml）
fn shader_categories() -> Vec<CategoryTag> {
    vec![
        CategoryTag::from_combined("6553/", "写实风"),
        CategoryTag::from_combined("6554/", "幻想风"),
        CategoryTag::from_combined("6555/", "原版风"),
    ]
}

/// 数据包分类（参考 PCL2 PageDownloadDataPack.xaml）
fn datapack_categories() -> Vec<CategoryTag> {
    vec![
        CategoryTag::from_combined("6948/", "冒险"),
        CategoryTag::from_combined("6949/", "幻想"),
        CategoryTag::from_combined("6950/", "支持库"),
        CategoryTag::from_combined("6952/", "魔法"),
        CategoryTag::from_combined("6946/", "Mod相关"),
        CategoryTag::from_combined("6951/", "科技"),
        CategoryTag::from_combined("6953/", "实用"),
    ]
}
