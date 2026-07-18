use super::*;

#[test]
fn identifiers_and_locales_are_normalized_and_validated() {
    assert_eq!(
        AppId::new("com.example.app").unwrap().as_str(),
        "com.example.app"
    );
    assert!(AppId::new("com.example bad").is_err());
    assert_eq!(Language::new("PT-BR").unwrap().as_str(), "pt-br");
    assert!(Language::new("pt--br").is_err());
    assert_eq!(Country::new("EE").unwrap().as_str(), "ee");
    assert!(Country::new("est").is_err());
}

#[test]
fn forward_compatible_enums_round_trip_through_serde() {
    let collection = Collection::custom("future_collection").unwrap();
    let encoded = serde_json::to_string(&collection).unwrap();
    assert_eq!(encoded, "\"future_collection\"");
    assert_eq!(
        serde_json::from_str::<Collection>(&encoded).unwrap(),
        collection
    );
    assert!(Collection::custom("\n").is_err());
}

#[test]
fn money_and_tokens_keep_exact_and_opaque_values() {
    let money = Money::new(1_234_567, Some("EUR".into()), Some("€1.23".into()));
    assert_eq!(money.micros, 1_234_567);
    assert!(!money.is_free());
    let token = PageToken::new("secret-continuation").unwrap();
    assert_eq!(token.expose(), "secret-continuation");
    assert_eq!(format!("{token:?}"), "PageToken(len=19)");
    assert!(!format!("{token:?}").contains(token.expose()));
}

#[test]
fn unknown_fields_do_not_break_non_exhaustive_model_deserialization() {
    let value = serde_json::json!({
        "app_id": "com.example.app",
        "title": "Example App",
        "store_url": "https://play.google.com/store/apps/details?id=com.example.app",
        "icon_url": null,
        "developer": null,
        "developer_id": null,
        "score": null,
        "score_text": null,
        "price": null,
        "is_free": true,
        "summary": null,
        "future_field": [1, 2, 3]
    });
    let app: AppOverview = serde_json::from_value(value).unwrap();
    assert_eq!(app.app_id.as_str(), "com.example.app");
}
