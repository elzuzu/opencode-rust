use opencode_rust::util::config::Info;

#[test]
fn test_deserialize_empty_info() {
    let json = "{}";
    let info: Info = serde_json::from_str(json).unwrap();
    assert!(info.schema.is_none());
    assert!(info.theme.is_none());
}

#[test]
fn test_deserialize_info_with_theme() {
    let json = r#"{"theme": "dark"}"#;
    let info: Info = serde_json::from_str(json).unwrap();
    assert_eq!(info.theme, Some("dark".to_string()));
}
