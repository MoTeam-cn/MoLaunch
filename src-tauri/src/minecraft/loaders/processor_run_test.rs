//! processor 执行器单测（只测纯逻辑，不起子进程）

use serde_json::json;

use super::{outputs_of, plan_client_processors};

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

#[test]
fn test_outputs_of_单输出() {
    let outputs = outputs_of(&args(&["--task", "decompile", "--output", "/out/a.jar"]));
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0], std::path::PathBuf::from("/out/a.jar"));
}

#[test]
fn test_outputs_of_多输出要全部收下() {
    let outputs = outputs_of(&args(&[
        "--outputs",
        "/out/a.jar",
        "/out/b.jar",
        "/out/c.jar",
        "--other",
        "x",
    ]));
    assert_eq!(
        outputs.len(),
        3,
        "早先只取第一个，导致 b/c 没生成也会被判定为齐了: {:?}",
        outputs
    );
    assert_eq!(outputs[1], std::path::PathBuf::from("/out/b.jar"));
}

#[test]
fn test_outputs_of_遇到下一个选项就停() {
    let outputs = outputs_of(&args(&["--outputs", "/out/a.jar", "--verbose"]));
    assert_eq!(outputs.len(), 1);
}

#[test]
fn test_outputs_of_没有声明时为空() {
    assert!(outputs_of(&args(&["--task", "decompile"])).is_empty());
}

#[test]
fn test_outputs_of_结尾孤立参数不越界() {
    assert!(outputs_of(&args(&["--output"])).is_empty());
    assert!(outputs_of(&args(&["--outputs"])).is_empty());
}

#[test]
fn test_plan_client_processors_缺_jar_直接报错() {
    let profile = json!({
        "processors": [
            { "sides": ["client"], "jar": "net.minecraft:client:26.3-1:srg", "args": [] }
        ]
    });
    let result = plan_client_processors(&profile, std::path::Path::new("C:/mc"), "26.3");
    assert!(
        result.is_err(),
        "缺 processor jar 说明库没下全，跳过会让安装看起来成功却少跑一步"
    );
    let message = result.err().map(|e| e.to_string()).unwrap_or_default();
    assert!(message.contains("缺少 processor jar"), "实际: {}", message);
}

#[test]
fn test_plan_client_processors_空profile返回空() {
    let profile = json!({});
    let planned = plan_client_processors(&profile, std::path::Path::new("C:/mc"), "26.3")
        .expect("空 profile 不该报错");
    assert!(planned.is_empty());
}

#[test]
fn test_plan_client_processors_缺少jar字段时报错() {
    let profile = json!({ "processors": [{ "args": ["--task", "x"] }] });
    assert!(plan_client_processors(&profile, std::path::Path::new("C:/mc"), "26.3").is_err());
}
