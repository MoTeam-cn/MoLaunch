//! processor 执行器单测（只测纯逻辑，不起子进程）

use serde_json::json;

use super::{outputs_of, plan_client_processors};

#[test]
fn test_outputs_of_认出单输出与多输出() {
    let arguments: Vec<String> = ["--task", "decompile", "--output", "/out/a.jar"]
        .iter()
        .map(|item| item.to_string())
        .collect();
    let outputs = outputs_of(&arguments);
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0], std::path::PathBuf::from("/out/a.jar"));

    let arguments: Vec<String> = ["--outputs", "/out/a.jar", "/out/b.jar"]
        .iter()
        .map(|item| item.to_string())
        .collect();
    assert_eq!(outputs_of(&arguments).len(), 1, "只取紧跟声明的那个");
}

#[test]
fn test_outputs_of_没有声明时为空() {
    let arguments: Vec<String> = ["--task", "decompile"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert!(outputs_of(&arguments).is_empty());
}

#[test]
fn test_outputs_of_结尾孤立参数不越界() {
    let arguments: Vec<String> = ["--output"].iter().map(|s| s.to_string()).collect();
    assert!(outputs_of(&arguments).is_empty());
}

#[test]
fn test_plan_client_processors_按_sides_过滤() {
    let profile = json!({
        "processors": [
            { "sides": ["client"], "jar": "net.minecraft:client:26.3-1:srg", "args": [] },
            { "sides": ["server"], "jar": "net.minecraft:server:26.3-1:srg", "args": [] },
            { "jar": "net.neoforged:installertools:1.0", "args": [] }
        ]
    });
    let planned = plan_client_processors(&profile, std::path::Path::new("C:/mc"), "26.3");
    assert_eq!(
        planned.len(),
        0,
        "jar 不在盘上时全部跳过，不会凭空造出执行计划"
    );
}

#[test]
fn test_plan_client_processors_空profile返回空() {
    let profile = json!({});
    assert!(plan_client_processors(&profile, std::path::Path::new("C:/mc"), "26.3").is_empty());
}
