//! 原版版本号提取（多策略：inheritsFrom → --fml.mcVersion → downloads URL → jar → id 正则）

/// 提取原版版本号（pub(crate) 供 commands 层复用）
pub(crate) fn extract_original_version(
    json: &serde_json::Value,
    _json_content: &str,
) -> Option<String> {
    // 策略1: 从 inheritsFrom 获取
    if let Some(inherits) = json["inheritsFrom"].as_str() {
        return Some(inherits.to_string());
    }

    // 策略2: 从 arguments 中的 --fml.mcVersion 获取
    if let Some(arguments) = json["arguments"].as_object() {
        if let Some(game) = arguments.get("game").and_then(|g| g.as_array()) {
            for (i, arg) in game.iter().enumerate() {
                if let Some(arg_str) = arg.as_str() {
                    if arg_str == "--fml.mcVersion" {
                        if let Some(next_arg) = game.get(i + 1) {
                            if let Some(version) = next_arg.as_str() {
                                return Some(version.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // 策略3: 从 downloads URL 中提取
    if let Some(downloads) = json["downloads"].as_object() {
        if let Some(client) = downloads.get("client") {
            if let Some(url) = client["url"].as_str() {
                static RE_URL: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
                let re =
                    RE_URL.get_or_init(|| regex::Regex::new(r"(\d+\.\d+(\.\d+)?(-\w+)?)").unwrap());
                if let Some(captures) = re.captures(url) {
                    return Some(captures[1].to_string());
                }
            }
        }
    }

    // 策略4: 从 jar 字段获取
    if let Some(jar) = json["jar"].as_str() {
        return Some(jar.to_string());
    }

    // 策略5: 从 id 字段正则匹配
    if let Some(id) = json["id"].as_str() {
        static RE_ID: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let re = RE_ID.get_or_init(|| regex::Regex::new(r"^(\d+\.\d+(\.\d+)?)").unwrap());
        if let Some(captures) = re.captures(id) {
            return Some(captures[1].to_string());
        }
    }

    None
}
