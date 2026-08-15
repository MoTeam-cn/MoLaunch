//! CreateRoomRequest 序列化契约测试（对齐 api-server snake_case DTO）

use super::CreateRoomRequest;

#[test]
fn create_room_request_serializes_snake_case() {
    let req = CreateRoomRequest {
        room_code: "U/SSV7-13KH-GSM5-G2PB".to_string(),
        remark: String::new(),
        is_public: true,
        password: String::new(),
        host_mc_version: "1.21.4".to_string(),
        host_mc_port: 25565,
        host_loader: None,
        host_loader_version: None,
        modpack: None,
    };
    let value = serde_json::to_value(&req).unwrap();
    let obj = value.as_object().unwrap();
    assert_eq!(obj["room_code"], "U/SSV7-13KH-GSM5-G2PB");
    assert_eq!(obj["is_public"], true);
    assert_eq!(obj["host_mc_version"], "1.21.4");
    assert_eq!(obj["host_mc_port"], 25565);
    assert!(!obj.contains_key("password"));
    assert!(!obj.contains_key("modpack"));
}
