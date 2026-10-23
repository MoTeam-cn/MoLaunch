//! 启动参数自检单元测试

use super::validate_jvm_args;

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_validate_jvm_args_正常参数通过() {
    let jvm = args(&["-Xmx4G", "-cp", "a.jar;b.jar", "-Djava.library.path=n"]);
    assert!(validate_jvm_args(&jvm).is_ok());
}

#[test]
fn test_validate_jvm_args_缺少取值报错() {
    let jvm = args(&["-Xmx4G", "-cp"]);
    let err = validate_jvm_args(&jvm).expect_err("裸 -cp 必须被拦截");
    assert!(err.contains("-cp"));
}

#[test]
fn test_validate_jvm_args_空取值报错() {
    let jvm = args(&["-cp", "", "-Xmx1G"]);
    assert!(validate_jvm_args(&jvm).is_err());
}

#[test]
fn test_validate_jvm_args_取值是另一个选项时报错() {
    let jvm = args(&["-cp", "-Xmx1G"]);
    assert!(validate_jvm_args(&jvm).is_err());
}

#[test]
fn test_validate_jvm_args_多个别名都能识别() {
    assert!(validate_jvm_args(&args(&["-classpath"])).is_err());
    assert!(validate_jvm_args(&args(&["--class-path"])).is_err());
    assert!(validate_jvm_args(&args(&["--class-path", "a.jar"])).is_ok());
}
