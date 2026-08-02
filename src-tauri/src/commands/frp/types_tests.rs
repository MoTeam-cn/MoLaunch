use crate::commands::frp::types::FieldMapping;
use std::collections::HashMap;

#[test]
fn field_mapping_three_forms() {
    let fields: HashMap<String, FieldMapping> = serde_json::from_str(
        r#"{
            "id": "id",
            "serverHost": { "field": "connectAddress", "split": ":" },
            "token": "{account.token}"
        }"#,
    )
    .expect("三种形式的 FieldMapping 均能解析");

    assert_eq!(fields["id"].field.as_deref(), Some("id"));
    assert_eq!(fields["id"].split, None);
    assert_eq!(fields["id"].value, None);

    assert_eq!(
        fields["serverHost"].field.as_deref(),
        Some("connectAddress")
    );
    assert_eq!(fields["serverHost"].split.as_deref(), Some(":"));
    assert_eq!(fields["serverHost"].value, None);

    assert_eq!(fields["token"].value.as_deref(), Some("{account.token}"));
    assert_eq!(fields["token"].field, None);
    assert_eq!(fields["token"].split, None);
}
