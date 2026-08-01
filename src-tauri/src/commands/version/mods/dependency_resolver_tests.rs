//! dependency_resolver 单元测试

use super::*;
use crate::minecraft::community::types::ModLoaders;

fn make_version(id: &str, gv: &[&str], ml: u32, rt: ReleaseType, date: &str) -> ResourceVersion {
    ResourceVersion {
        id: id.to_string(),
        display: String::new(),
        version: String::new(),
        release_date: date.to_string(),
        download_count: 0,
        mod_loaders: ml,
        game_versions: gv.iter().map(|s| s.to_string()).collect(),
        release_type: rt,
        file_name: String::new(),
        download_url: String::new(),
        hash: None,
        size: 0,
        dependencies: Vec::new(),
    }
}

#[test]
fn test_pick_best_version_prefers_release() {
    let versions = vec![
        make_version(
            "a",
            &["1.20.1"],
            ModLoaders::FORGE,
            ReleaseType::Alpha,
            "2023-01-01",
        ),
        make_version(
            "b",
            &["1.20.1"],
            ModLoaders::FORGE,
            ReleaseType::Release,
            "2023-06-01",
        ),
        make_version(
            "c",
            &["1.20.1"],
            ModLoaders::FORGE,
            ReleaseType::Beta,
            "2023-12-01",
        ),
    ];
    let best = pick_best_version(&versions, "1.20.1", ModLoaders::FORGE);
    assert_eq!(best.unwrap().id, "b");
}

#[test]
fn test_pick_best_version_filters_game_version() {
    let versions = vec![
        make_version(
            "a",
            &["1.19.2"],
            ModLoaders::FORGE,
            ReleaseType::Release,
            "2023-06-01",
        ),
        make_version(
            "b",
            &["1.20.1"],
            ModLoaders::FORGE,
            ReleaseType::Release,
            "2023-06-01",
        ),
    ];
    let best = pick_best_version(&versions, "1.20.1", ModLoaders::FORGE);
    assert_eq!(best.unwrap().id, "b");
}

#[test]
fn test_pick_best_version_filters_loader() {
    let versions = vec![
        make_version(
            "a",
            &["1.20.1"],
            ModLoaders::FABRIC,
            ReleaseType::Release,
            "2023-06-01",
        ),
        make_version(
            "b",
            &["1.20.1"],
            ModLoaders::FORGE,
            ReleaseType::Release,
            "2023-06-01",
        ),
    ];
    let best = pick_best_version(&versions, "1.20.1", ModLoaders::FORGE);
    assert_eq!(best.unwrap().id, "b");
}

#[test]
fn test_pick_best_version_no_compatible() {
    let versions = vec![make_version(
        "a",
        &["1.19.2"],
        ModLoaders::FORGE,
        ReleaseType::Release,
        "2023-06-01",
    )];
    let best = pick_best_version(&versions, "1.20.1", ModLoaders::FORGE);
    assert!(best.is_none());
}

#[test]
fn test_is_project_installed_by_slug() {
    let mut installed = HashSet::new();
    installed.insert("jei".to_string());
    let project = ResourceProject {
        platform: Platform::CurseForge,
        resource_type: ResourceType::Mod,
        id: "12345".to_string(),
        slug: "jei".to_string(),
        raw_name: "Just Enough Items".to_string(),
        translated_name: String::new(),
        description: String::new(),
        website: String::new(),
        last_update: String::new(),
        download_count: 0,
        mod_loaders: 0,
        tags: Vec::new(),
        logo_url: None,
        game_versions: Vec::new(),
    };
    assert!(is_project_installed(&project, &installed));
}

#[test]
fn test_is_project_installed_by_id() {
    let mut installed = HashSet::new();
    installed.insert("p7dr8msh".to_string());
    let project = ResourceProject {
        platform: Platform::Modrinth,
        resource_type: ResourceType::Mod,
        id: "P7dR8mSH".to_string(),
        slug: "fabric-api".to_string(),
        raw_name: "Fabric API".to_string(),
        translated_name: String::new(),
        description: String::new(),
        website: String::new(),
        last_update: String::new(),
        download_count: 0,
        mod_loaders: 0,
        tags: Vec::new(),
        logo_url: None,
        game_versions: Vec::new(),
    };
    assert!(is_project_installed(&project, &installed));
}

#[test]
fn test_is_project_installed_not_found() {
    let installed = HashSet::new();
    let project = ResourceProject {
        platform: Platform::CurseForge,
        resource_type: ResourceType::Mod,
        id: "99999".to_string(),
        slug: "unknown-mod".to_string(),
        raw_name: "Unknown".to_string(),
        translated_name: String::new(),
        description: String::new(),
        website: String::new(),
        last_update: String::new(),
        download_count: 0,
        mod_loaders: 0,
        tags: Vec::new(),
        logo_url: None,
        game_versions: Vec::new(),
    };
    assert!(!is_project_installed(&project, &installed));
}
