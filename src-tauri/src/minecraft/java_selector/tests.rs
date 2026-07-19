//! Java 选择算法测试

use crate::minecraft::java::JavaRuntime;

use super::compat::check_java_compatible;
use super::rules::{get_recommended_java_version, get_required_java_version};
use super::select::{select_best_java, select_best_java_with_loader};

#[test]
fn test_get_required_java_version() {
    // 1.20.5+ -> Java 21
    assert_eq!(get_required_java_version("1.20.5"), 21);
    assert_eq!(get_required_java_version("1.20.6"), 21);
    assert_eq!(get_required_java_version("1.21"), 21);
    assert_eq!(get_required_java_version("1.21.1"), 21);

    // 1.18-1.20.4 -> Java 17
    assert_eq!(get_required_java_version("1.18"), 17);
    assert_eq!(get_required_java_version("1.18.2"), 17);
    assert_eq!(get_required_java_version("1.19.4"), 17);
    assert_eq!(get_required_java_version("1.20.4"), 17);

    // 1.17 -> Java 16
    assert_eq!(get_required_java_version("1.17"), 16);
    assert_eq!(get_required_java_version("1.17.1"), 16);

    // 1.12-1.16 -> Java 8
    assert_eq!(get_required_java_version("1.12"), 8);
    assert_eq!(get_required_java_version("1.12.2"), 8);
    assert_eq!(get_required_java_version("1.16.5"), 8);

    // 1.5 以下 -> Java 8
    assert_eq!(get_required_java_version("1.5"), 8);
    assert_eq!(get_required_java_version("1.4.7"), 8);
}

#[test]
fn test_get_recommended_java_version() {
    assert_eq!(get_recommended_java_version("1.20.5"), 21);
    assert_eq!(get_recommended_java_version("1.18.2"), 17);
    assert_eq!(get_recommended_java_version("1.17"), 17); // 推荐 17 而非 16
    assert_eq!(get_recommended_java_version("1.16.5"), 8);
}

#[test]
fn test_select_best_java_with_user_path() {
    let java_list = vec![
        JavaRuntime {
            executable: "C:\\Java8\\java.exe".to_string(),
            path_folder: "C:\\Java8".to_string(),
            is_user_import: false,
            version: "1.8.0_321".to_string(),
            major_version: 8,
            is_jre: true,
            is_64bit: true,
        },
        JavaRuntime {
            executable: "C:\\Java17\\java.exe".to_string(),
            path_folder: "C:\\Java17".to_string(),
            is_user_import: false,
            version: "17.0.2".to_string(),
            major_version: 17,
            is_jre: true,
            is_64bit: true,
        },
    ];

    // 用户指定 Java 8
    let result = select_best_java("1.16.5", &java_list, Some("C:\\Java8\\java.exe"));
    assert_eq!(result, Some("C:\\Java8\\java.exe".to_string()));

    // 用户指定的 Java 不满足要求（MC 1.20.5 需要 Java 21，但列表中没有）
    let result = select_best_java("1.20.5", &java_list, Some("C:\\Java8\\java.exe"));
    assert_eq!(result, None); // 没有满足要求的 Java
}

#[test]
fn test_select_best_java_auto() {
    let java_list = vec![
        JavaRuntime {
            executable: "C:\\Java8\\java.exe".to_string(),
            path_folder: "C:\\Java8".to_string(),
            is_user_import: false,
            version: "1.8.0_321".to_string(),
            major_version: 8,
            is_jre: true,
            is_64bit: true,
        },
        JavaRuntime {
            executable: "C:\\Java17\\java.exe".to_string(),
            path_folder: "C:\\Java17".to_string(),
            is_user_import: false,
            version: "17.0.2".to_string(),
            major_version: 17,
            is_jre: true,
            is_64bit: true,
        },
        JavaRuntime {
            executable: "C:\\Java21\\java.exe".to_string(),
            path_folder: "C:\\Java21".to_string(),
            is_user_import: false,
            version: "21.0.1".to_string(),
            major_version: 21,
            is_jre: true,
            is_64bit: true,
        },
    ];

    // MC 1.16.5 需要 Java 8
    let result = select_best_java("1.16.5", &java_list, None);
    assert_eq!(result, Some("C:\\Java8\\java.exe".to_string()));

    // MC 1.18.2 需要 Java 17
    let result = select_best_java("1.18.2", &java_list, None);
    assert_eq!(result, Some("C:\\Java17\\java.exe".to_string()));

    // MC 1.20.5 需要 Java 21
    let result = select_best_java("1.20.5", &java_list, None);
    assert_eq!(result, Some("C:\\Java21\\java.exe".to_string()));
}

#[test]
fn test_select_best_java_with_loader_smoke() {
    // 简单冒烟测试：带 loader 调用不应 panic，且与无 loader 时结果一致（原版规则）
    let java_list = vec![JavaRuntime {
        executable: "C:\\Java17\\java.exe".to_string(),
        path_folder: "C:\\Java17".to_string(),
        is_user_import: false,
        version: "17.0.2".to_string(),
        major_version: 17,
        is_jre: true,
        is_64bit: true,
    }];

    let result = select_best_java_with_loader("1.18.2", Some("fabric"), &java_list, None);
    assert_eq!(result, Some("C:\\Java17\\java.exe".to_string()));
}

#[test]
fn test_check_java_compatible() {
    // Java 8 兼容 MC 1.16.5
    assert!(check_java_compatible(8, "1.16.5", None).is_ok());
    // Java 8 不兼容 MC 1.18（需要 17+）
    assert!(check_java_compatible(8, "1.18", None).is_err());
    // Java 21 兼容 MC 1.20.5
    assert!(check_java_compatible(21, "1.20.5", None).is_ok());
}
