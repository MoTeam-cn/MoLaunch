//! Mod 索引测试

use super::{load_mod_index, split_jar_name};
use std::path::Path;

#[test]
fn test_split_jar_name_取第一个连字符前的部分作为_modid() {
    let (modid, display) = split_jar_name("create-1.20.1-0.5.1");
    assert_eq!(modid, "create");
    assert_eq!(display, "create-1.20.1-0.5.1");
}

#[test]
fn test_split_jar_name_无连字符时整体作为_modid() {
    let (modid, _display) = split_jar_name("sodium");
    assert_eq!(modid, "sodium");
}

#[test]
fn test_load_mod_index_目录不存在时返回空() {
    let index = load_mod_index(Path::new("C:/__不存在的_mods_目录__"));
    assert!(index.is_empty());
}
