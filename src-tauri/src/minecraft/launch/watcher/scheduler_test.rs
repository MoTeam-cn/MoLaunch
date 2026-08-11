use super::parse_lan_port;

#[test]
fn parse_lan_port_matches_common_formats() {
    assert_eq!(
        parse_lan_port("[16:34:49] [Client thread/INFO]: Started on 4053"),
        Some(4053)
    );
    assert_eq!(
        parse_lan_port("[Server thread/INFO]: Local game hosted on port 49152"),
        Some(49152)
    );
    assert_eq!(
        parse_lan_port("[Server thread/INFO]: Published server on 192.168.1.100:49152"),
        Some(49152)
    );
    assert_eq!(
        parse_lan_port("[Server thread/INFO]: Started serving on 192.168.1.100:25565"),
        Some(25565)
    );
}

#[test]
fn parse_lan_port_ignores_unrelated_lines() {
    assert_eq!(
        parse_lan_port(r#"[Server thread/INFO]: Preparing level "world""#),
        None
    );
    assert_eq!(
        parse_lan_port("[Client thread/INFO]: Started on world gen"),
        None
    );
    assert_eq!(parse_lan_port(""), None);
}
