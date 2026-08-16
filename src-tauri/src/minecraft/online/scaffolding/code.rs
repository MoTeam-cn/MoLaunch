//! Scaffolding 联机房间码：生成 / 解析 / 公开标识。
//!
//! 房间码形如 `U/NNNN-NNNN-SSSS-SSSS`：前两段为网络名标识 N，后两段为网络密钥 S。
//! 校验规则：字符按 0-9、A-H、J-N、P-Z（含 L）映射到 [0, 34) 后，按小端序读得的整型应能被 7 整除。

use rand::RngCore;

/// 房间码字符集（剔除易混淆的 I / O，保留 L；与 Terracotta 标准一致，共 34 字符）
const CHARSET: &[u8] = b"0123456789ABCDEFGHJKLMNPQRSTUVWXYZ";

/// 网络名前缀
const NETWORK_NAME_PREFIX: &str = "scaffolding-mc-";

/// 字符映射为数值，非法字符返回 None
fn char_to_value(c: u8) -> Option<u8> {
    CHARSET.iter().position(|&x| x == c).map(|i| i as u8)
}

/// 小端序 base-34 整型模 7 校验（模运算避免溢出）
fn validate_checksum(chars: &[u8]) -> bool {
    if chars.len() != 16 {
        return false;
    }
    let base_mod = CHARSET.len() % 7;
    let mut pow = 1usize;
    let mut acc = 0usize;
    for &c in chars {
        let Some(v) = char_to_value(c) else {
            return false;
        };
        acc = (acc + (v as usize % 7) * pow) % 7;
        pow = (pow * base_mod) % 7;
    }
    acc == 0
}

/// 生成符合校验规则的房间码 `U/NNNN-NNNN-SSSS-SSSS`
pub fn generate_room_code() -> String {
    let mut rng = rand::thread_rng();
    loop {
        let mut idx = [0u8; 16];
        rng.fill_bytes(&mut idx);
        let mut chars = [0u8; 16];
        for (i, &n) in idx.iter().enumerate() {
            chars[i] = CHARSET[n as usize % CHARSET.len()];
        }
        if !validate_checksum(&chars) {
            continue;
        }
        return format!(
            "U/{}{}{}{}-{}{}{}{}-{}{}{}{}-{}{}{}{}",
            chars[0] as char,
            chars[1] as char,
            chars[2] as char,
            chars[3] as char,
            chars[4] as char,
            chars[5] as char,
            chars[6] as char,
            chars[7] as char,
            chars[8] as char,
            chars[9] as char,
            chars[10] as char,
            chars[11] as char,
            chars[12] as char,
            chars[13] as char,
            chars[14] as char,
            chars[15] as char,
        );
    }
}

/// 解析房间码，返回 (network_name, network_secret)
///
/// 仅做格式校验（前缀 / 长度 / 分隔符 / 字符集），不做校验和检查，
/// 以保证对官方 34 字符集生成的房间码同样可解析。
pub fn parse(room_code: &str) -> Result<(String, String), String> {
    let trimmed = room_code.trim();
    if !trimmed.starts_with("U/") && !trimmed.starts_with("u/") {
        return Err("房间码必须以 U/ 开头".to_string());
    }
    let code = &trimmed[2..];
    if code.len() != 19 {
        return Err("房间码长度不正确".to_string());
    }
    if code.as_bytes()[4] != b'-' || code.as_bytes()[9] != b'-' || code.as_bytes()[14] != b'-' {
        return Err("房间码分隔符不正确".to_string());
    }
    let parts: Vec<&str> = code.split('-').collect();
    if parts.len() != 4 {
        return Err("房间码段数不正确".to_string());
    }
    for part in &parts {
        if part.len() != 4 {
            return Err("房间码段长度不正确".to_string());
        }
        for &b in part.as_bytes() {
            if char_to_value(b).is_none() {
                return Err("房间码包含非法字符".to_string());
            }
        }
    }
    let network_name = format!("{NETWORK_NAME_PREFIX}{}-{}", parts[0], parts[1]);
    let network_secret = format!("{}-{}", parts[2], parts[3]);
    Ok((network_name, network_secret))
}

/// 公开标识：仅暴露 N 段（不含密钥），用于大厅展示与去重
pub fn public_identifier(room_code: &str) -> String {
    parse(room_code)
        .map(|(name, _)| name.trim_start_matches(NETWORK_NAME_PREFIX).to_string())
        .map(|n| format!("U/{n}"))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_example() {
        let (network_name, network_secret) = parse("U/YNZE-U61D-2206-HXRG").unwrap();
        assert_eq!(network_name, "scaffolding-mc-YNZE-U61D");
        assert_eq!(network_secret, "2206-HXRG");
    }

    #[test]
    fn test_generate_is_valid() {
        for _ in 0..16 {
            let code = generate_room_code();
            assert!(code.starts_with("U/"));
            assert_eq!(code.len(), 21);
            let (name, secret) = parse(&code).unwrap();
            assert!(name.starts_with(NETWORK_NAME_PREFIX));
            assert!(!secret.is_empty());
        }
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse("").is_err());
        assert!(parse("X/ABCD-EFGH-JKMN-PQRS").is_err());
        assert!(parse("U/ABCD-EFGH-JKMN-PQRS-EXTRA").is_err());
        assert!(parse("U/ABC-EFGH-JKMN-PQRS").is_err());
        assert!(parse("U/ABC!-EFGH-JKMN-PQRS").is_err());
    }

    #[test]
    fn test_public_identifier() {
        assert_eq!(public_identifier("U/YNZE-U61D-2206-HXRG"), "U/YNZE-U61D");
    }
}
