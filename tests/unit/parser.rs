use super::*;

#[test]
fn extracts_script_json_without_evaluating_javascript() {
    let html = r#"<script>AF_initDataCallback({key: 'ds:5', hash: 'x', data:[1,{"ok":true}], sideChannel: {}});</script>"#;
    let data = parse_html_data(html).unwrap();
    assert_eq!(data["ds:5"][1]["ok"], true);
}

#[test]
fn parses_clean_and_chunked_rpc_envelopes() {
    let clean = r#"[["wrb.fr","RPC","[1,2]",null]]"#;
    assert_eq!(
        parse_rpc_response(clean, "RPC").unwrap(),
        serde_json::json!([1, 2])
    );
    let chunked = format!(")]}}'\n12\n{clean}\n");
    assert_eq!(
        parse_rpc_response(&chunked, "RPC").unwrap(),
        serde_json::json!([1, 2])
    );
}

#[test]
fn malformed_input_is_an_error_not_a_panic() {
    assert!(parse_rpc_response("not json", "RPC").is_err());
    assert!(
        parse_html_data("<html><script>broken</script></html>")
            .unwrap()
            .is_empty()
    );
}

proptest::proptest! {
    #[test]
    fn arbitrary_text_never_panics_html_or_rpc_parsers(input in ".{0,2048}") {
        let _ = parse_html_data(&input);
        let _ = parse_rpc_response(&input, "RPC");
    }
}

#[test]
fn scalar_extractors_handle_localized_and_invalid_values() {
    let value = serde_json::json!([["1,234+", "4,7", 1, -1, "https://example.invalid"]]);
    assert_eq!(unsigned(&value, &[0, 0]), Some(1234));
    assert_eq!(float(&value, &[0, 1]), Some(4.7));
    assert_eq!(boolean(&value, &[0, 2]), Some(true));
    assert_eq!(unsigned(&value, &[0, 3]), None);
    assert_eq!(
        url(&value, &[0, 4]).unwrap().host_str(),
        Some("example.invalid")
    );
    assert_eq!(text(&value, &[99]), None);
}
