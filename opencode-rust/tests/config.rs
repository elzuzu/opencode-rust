use opencode_rust::config::Info;

#[test]
fn test_deserialize_empty_config() {
    let json = "{}";
    let config: Result<Info, _> = serde_json::from_str(json);
    assert!(config.is_ok());
}
