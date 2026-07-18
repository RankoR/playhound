//! Black-box tests for the fixture sanitizer.

use playhound_xtask::{FixtureKind, sanitize};

#[test]
fn replaces_identity_and_media_fields() {
    let input = serde_json::json!({
        "app_id": "org.real.application",
        "title": "Real title",
        "developer": "Real developer",
        "icon_url": "https://real-images.example/image.png",
        "store_url": "https://play.google.com/store/apps/details?id=org.real.application"
    });
    let output = sanitize(FixtureKind::App, input).unwrap();
    assert_eq!(output["app_id"], "com.example.app");
    assert_eq!(output["title"], "Example App");
    assert_eq!(output["icon_url"], "https://example.invalid/image.png");
    assert!(
        output["store_url"]
            .as_str()
            .unwrap()
            .contains("com.example.app")
    );
}

#[test]
fn replaces_bare_suggestions() {
    let output = sanitize(
        FixtureKind::Suggestions,
        serde_json::json!(["real product", "real company"]),
    )
    .unwrap();
    assert_eq!(
        output,
        serde_json::json!(["example suggestion", "example suggestion"])
    );
}

#[test]
fn replaces_review_identity_text_and_version_metadata() {
    let input = serde_json::json!({
        "id": "real-review-id",
        "user_name": "Real User",
        "title": "Real review title",
        "text": "Real review body",
        "app_version": "99.0.123",
        "date": "2026-07-18T12:00:00Z"
    });
    let output = sanitize(FixtureKind::Reviews, input).unwrap();
    assert_eq!(output["id"], "example-review-id");
    assert_eq!(output["user_name"], "Example User");
    assert_eq!(output["title"], "Example Review");
    assert_eq!(output["text"], "Example review text.");
    assert_eq!(output["app_version"], "1.2.3");
    assert_eq!(output["date"], "2026-01-02T03:04:05Z");
}
