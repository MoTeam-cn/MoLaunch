//! lan_probe 单元测试

use super::parse_lan_broadcast;

#[test]
fn parse_valid_broadcast() {
    let (motd, port) =
        parse_lan_broadcast("[MOTD]MoLaunch 联机 A1B2[/MOTD][AD]25566[/AD]").unwrap();
    assert_eq!(motd, "MoLaunch 联机 A1B2");
    assert_eq!(port, 25566);
}

#[test]
fn parse_broadcast_standard_port() {
    let (motd, port) =
        parse_lan_broadcast("[MOTD]A Minecraft Server[/MOTD][AD]25565[/AD]").unwrap();
    assert_eq!(motd, "A Minecraft Server");
    assert_eq!(port, 25565);
}

#[test]
fn parse_invalid_broadcast() {
    assert!(parse_lan_broadcast("").is_none());
    assert!(parse_lan_broadcast("[MOTD]x[/MOTD]").is_none());
    assert!(parse_lan_broadcast("[MOTD]x[/MOTD][AD]abc[/AD]").is_none());
    assert!(parse_lan_broadcast("[MOTD]x[/MOTD][AD]99999[/AD]").is_none());
    assert!(parse_lan_broadcast("garbage").is_none());
}
