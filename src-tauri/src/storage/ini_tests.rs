//! ini 单元测试

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
    assert_eq!(
        ini.get("General", "game_dir"),
        Some(".minecraft".to_string())
    );
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
