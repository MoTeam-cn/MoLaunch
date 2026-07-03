//! INI 文件解析器
//!
//! 支持标准 INI 格式：
//! - [Section]
//! - key=value
//! - # 注释

/// INI 文件
#[derive(Debug, Clone)]
pub struct IniFile {
    /// 按顺序存储的段落
    sections: Vec<Section>,
}

/// 段落
#[derive(Debug, Clone)]
struct Section {
    name: String,
    keys: Vec<(String, String)>,
}

impl IniFile {
    /// 创建空的 INI 文件
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// 解析 INI 格式字符串
    pub fn parse(content: &str) -> Self {
        let mut sections = Vec::new();
        let mut current_section = Section {
            name: String::new(),
            keys: Vec::new(),
        };

        for line in content.lines() {
            let line = line.trim();
            
            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // 解析段落
            if line.starts_with('[') && line.ends_with(']') {
                // 保存当前段落
                if !current_section.name.is_empty() || !current_section.keys.is_empty() {
                    sections.push(current_section);
                }
                current_section = Section {
                    name: line[1..line.len()-1].trim().to_string(),
                    keys: Vec::new(),
                };
                continue;
            }

            // 解析键值对
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos+1..].trim().to_string();
                current_section.keys.push((key, value));
            }
        }

        // 保存最后一个段落
        if !current_section.name.is_empty() || !current_section.keys.is_empty() {
            sections.push(current_section);
        }

        Self { sections }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        
        for section in &self.sections {
            if !section.name.is_empty() {
                result.push_str(&format!("[{}]\n", section.name));
            }
            for (key, value) in &section.keys {
                result.push_str(&format!("{}={}\n", key, value));
            }
            result.push('\n');
        }
        
        result
    }

    /// 获取配置值
    pub fn get(&self, section: &str, key: &str) -> Option<String> {
        for s in &self.sections {
            if s.name == section {
                for (k, v) in &s.keys {
                    if k == key {
                        return Some(v.clone());
                    }
                }
            }
        }
        None
    }

    /// 获取配置值，带默认值
    pub fn get_or(&self, section: &str, key: &str, default: &str) -> String {
        self.get(section, key).unwrap_or_else(|| default.to_string())
    }

    /// 设置配置值
    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        // 查找段落
        for s in &mut self.sections {
            if s.name == section {
                // 查找键
                for (k, v) in &mut s.keys {
                    if k == key {
                        *v = value.to_string();
                        return;
                    }
                }
                // 键不存在，添加
                s.keys.push((key.to_string(), value.to_string()));
                return;
            }
        }
        // 段落不存在，创建新的
        self.sections.push(Section {
            name: section.to_string(),
            keys: vec![(key.to_string(), value.to_string())],
        });
    }

    /// 删除配置值
    pub fn remove(&mut self, section: &str, key: &str) {
        for s in &mut self.sections {
            if s.name == section {
                s.keys.retain(|(k, _)| k != key);
                return;
            }
        }
    }

    /// 删除段落
    pub fn remove_section(&mut self, section: &str) {
        self.sections.retain(|s| s.name != section);
    }

    /// 检查段落是否存在
    pub fn has_section(&self, section: &str) -> bool {
        self.sections.iter().any(|s| s.name == section)
    }

    /// 检查键是否存在
    pub fn has_key(&self, section: &str, key: &str) -> bool {
        self.get(section, key).is_some()
    }

    /// 获取段落的所有键值对
    pub fn get_section(&self, section: &str) -> Vec<(String, String)> {
        for s in &self.sections {
            if s.name == section {
                return s.keys.clone();
            }
        }
        Vec::new()
    }

    /// 获取所有段落名称
    pub fn sections(&self) -> Vec<String> {
        self.sections.iter().map(|s| s.name.clone()).collect()
    }
}

impl Default for IniFile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_serialize() {
        let input = r#"[General]
game_dir=.minecraft
theme=system

[Download]
max_threads=8
"#;
        let ini = IniFile::parse(input);
        assert_eq!(ini.get("General", "game_dir"), Some(".minecraft".to_string()));
        assert_eq!(ini.get("General", "theme"), Some("system".to_string()));
        assert_eq!(ini.get("Download", "max_threads"), Some("8".to_string()));
    }

    #[test]
    fn test_set_and_get() {
        let mut ini = IniFile::new();
        ini.set("Section", "key", "value");
        assert_eq!(ini.get("Section", "key"), Some("value".to_string()));
    }

    #[test]
    fn test_remove() {
        let mut ini = IniFile::new();
        ini.set("Section", "key", "value");
        ini.remove("Section", "key");
        assert_eq!(ini.get("Section", "key"), None);
    }
}
