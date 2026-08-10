use super::*;

#[test]
fn loader_lib_pattern_covers_main_loaders() {
    assert_eq!(loader_lib_pattern(&VersionType::Forge), Some("net.minecraftforge"));
    assert_eq!(loader_lib_pattern(&VersionType::NeoForge), Some("net.neoforged"));
    assert_eq!(loader_lib_pattern(&VersionType::Fabric), Some("net.fabricmc:fabric-loader"));
    assert_eq!(loader_lib_pattern(&VersionType::LiteLoader), Some("com.mumfrey:liteloader"));
    assert_eq!(loader_lib_pattern(&VersionType::Release), None);
    assert_eq!(loader_lib_pattern(&VersionType::Unknown), None);
}

#[test]
fn find_loader_lib_matches_pattern() {
    let json = serde_json::json!({
        "libraries": [
            {"name": "net.minecraft.client", "downloads": {}},
            {"name": "net.minecraftforge:forge:1.12.2-14.23.5.2860", "downloads": {}}
        ]
    });
    assert_eq!(
        find_loader_lib(&json, "net.minecraftforge"),
        Some("net.minecraftforge:forge:1.12.2-14.23.5.2860".to_string())
    );
    assert_eq!(find_loader_lib(&json, "com.mumfrey:liteloader"), None);
    assert_eq!(find_loader_lib(&serde_json::json!({"libraries": null}), "x"), None);
}

#[test]
fn json_lib_local_path_prefers_artifact_path() {
    let json = serde_json::json!({
        "libraries": [{
            "name": "net.minecraftforge:forge:1.12.2-14.23.5.2860",
            "downloads": {"artifact": {"path": "net/minecraftforge/forge/custom.jar"}}
        }]
    });
    let game_dir = Path::new("/tmp/mc");
    let p = json_lib_local_path(&json, "net.minecraftforge:forge:1.12.2-14.23.5.2860", game_dir);
    assert!(p.ends_with("net/minecraftforge/forge/custom.jar"));
}

#[test]
fn json_lib_local_path_falls_back_to_maven() {
    let json = serde_json::json!({
        "libraries": [{"name": "net.fabricmc:fabric-loader:0.15.11"}]
    });
    let game_dir = Path::new("/tmp/mc");
    let p = json_lib_local_path(&json, "net.fabricmc:fabric-loader:0.15.11", game_dir);
    assert!(p.ends_with("net/fabricmc/fabric-loader/0.15.11/fabric-loader-0.15.11.jar"));
}

#[test]
fn merge_libraries_dedup_same_name_replaced() {
    let mut target = serde_json::json!({
        "libraries": [
            {"name": "a:b:1"},
            {"name": "c:d:2"}
        ]
    });
    let fresh = serde_json::json!({
        "libraries": [
            {"name": "a:b:1", "downloads": {"artifact": {"path": "new.jar"}}},
            {"name": "e:f:3"}
        ]
    });
    merge_libraries_dedup(&mut target, &fresh);
    let libs = target["libraries"].as_array().unwrap();
    assert_eq!(libs.len(), 3);
    let a = libs.iter().find(|l| l["name"] == "a:b:1").unwrap();
    assert_eq!(a["downloads"]["artifact"]["path"], "new.jar");
}

#[test]
fn merge_libraries_dedup_creates_when_missing() {
    let mut target = serde_json::json!({"libraries": null});
    let fresh = serde_json::json!({"libraries": [{"name": "a:b:1"}]});
    merge_libraries_dedup(&mut target, &fresh);
    assert_eq!(target["libraries"].as_array().unwrap().len(), 1);
}

#[test]
fn merge_argument_arrays_appends_without_dup() {
    let mut target = serde_json::json!({
        "arguments": {
            "game": ["--a", "--b"],
            "jvm": ["-Xmx1G"]
        }
    });
    let fresh = serde_json::json!({
        "arguments": {
            "game": ["--b", "--c"],
            "jvm": ["-Xmx2G", "-Dforge"]
        }
    });
    merge_argument_arrays(&mut target, &fresh);
    let game = target["arguments"]["game"].as_array().unwrap();
    assert_eq!(game.len(), 3);
    let jvm = target["arguments"]["jvm"].as_array().unwrap();
    assert_eq!(jvm.len(), 3);
    assert!(jvm.contains(&serde_json::json!("-Xmx2G")));
}

#[test]
fn merge_minecraft_args_merges_tokens() {
    let mut target = serde_json::json!({"minecraftArguments": "--a --b"});
    let fresh = serde_json::json!({"minecraftArguments": "--b --c"});
    merge_minecraft_args(&mut target, &fresh);
    let args = target["minecraftArguments"].as_str().unwrap();
    assert!(args.contains("--a") && args.contains("--c"));
    assert_eq!(args.split(' ').count(), 3);
}

#[test]
fn merge_fields_skips_special_keys_and_keeps_id() {
    let mut target = serde_json::json!({
        "id": "keep-me",
        "mainClass": "old",
        "downloads": {"server": "s", "client": "old-client"},
        "libraries": [{"name": "x"}],
        "nested": {"a": 1, "b": 2}
    });
    let fresh = serde_json::json!({
        "id": "new-id",
        "mainClass": "new",
        "inheritsFrom": "1.12.2",
        "downloads": {"client": "new-client"},
        "nested": {"b": 3, "c": 4},
        "libraries": [{"name": "y"}]
    });
    merge_fields(&mut target, &fresh);
    assert_eq!(target["id"], "keep-me");
    assert_eq!(target["mainClass"], "new");
    assert_eq!(target["downloads"]["client"], "new-client");
    assert_eq!(target["downloads"]["server"], "s");
    assert!(target.get("inheritsFrom").is_none());
    assert_eq!(target["nested"]["a"], 1);
    assert_eq!(target["nested"]["b"], 3);
    assert_eq!(target["nested"]["c"], 4);
    // libraries 由 merge_libraries_dedup 单独处理
    assert_eq!(target["libraries"][0]["name"], "x");
}

#[test]
fn fresh_loader_dir_name_formats() {
    assert_eq!(
        fresh_loader_dir_name(&LoaderType::Forge, "1.12.2", "14.23.5.2860"),
        "1.12.2-forge-14.23.5.2860"
    );
    assert_eq!(
        fresh_loader_dir_name(&LoaderType::Fabric, "1.20.1", "0.15.11"),
        "fabric-0.15.11-1.20.1"
    );
    assert_eq!(
        fresh_loader_dir_name(&LoaderType::LiteLoader, "1.12.2", "1.12.2-SNAPSHOT"),
        "1.12.2-LiteLoader"
    );
}

#[test]
fn merge_loader_json_into_writes_merged_result() {
    let base = std::env::temp_dir().join(format!("repair_loader_test_{}", std::process::id()));
    let version_dir = base.join("versions").join("MyPack");
    std::fs::create_dir_all(&version_dir).unwrap();
    let existing = serde_json::json!({
        "id": "MyPack",
        "mainClass": "net.minecraft.launchwrapper.Launch",
        "minecraftArguments": "--a --b",
        "libraries": [{"name": "net.minecraft:launchwrapper:1.12"}]
    });
    std::fs::write(
        version_dir.join("MyPack.json"),
        serde_json::to_string_pretty(&existing).unwrap(),
    )
    .unwrap();

    let fresh_dir = base.join("versions").join("1.12.2-forge-14.23.5.2860");
    std::fs::create_dir_all(&fresh_dir).unwrap();
    let fresh = serde_json::json!({
        "id": "1.12.2-forge-14.23.5.2860",
        "mainClass": "net.minecraft.launchwrapper.Launch",
        "minecraftArguments": "--b --c",
        "libraries": [
            {"name": "net.minecraft:launchwrapper:1.12"},
            {"name": "net.minecraftforge:forge:1.12.2-14.23.5.2860"}
        ]
    });
    std::fs::write(
        fresh_dir.join("1.12.2-forge-14.23.5.2860.json"),
        serde_json::to_string_pretty(&fresh).unwrap(),
    )
    .unwrap();

    let result = merge_loader_json_into(&base, "MyPack", &existing, &fresh_dir);
    assert!(result.is_ok(), "merge failed: {:?}", result.err());

    let saved: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(version_dir.join("MyPack.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(saved["id"], "MyPack");
    assert!(saved.get("inheritsFrom").is_none());
    let libs = saved["libraries"].as_array().unwrap();
    assert_eq!(libs.len(), 2);
    assert!(libs.iter().any(|l| l["name"] == "net.minecraftforge:forge:1.12.2-14.23.5.2860"));
    let args = saved["minecraftArguments"].as_str().unwrap();
    assert!(args.contains("--a") && args.contains("--c"));

    std::fs::remove_dir_all(&base).ok();
}
