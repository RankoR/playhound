//! Contract checks for sanitized normalized snapshots captured from live responses.

use playhound::{AppDetails, AppOverview, Review};

#[test]
fn normalized_live_snapshots_deserialize_into_public_models() {
    let app: AppDetails =
        serde_json::from_str(include_str!("fixtures/live/app.json")).expect("app fixture");
    let search: Vec<AppOverview> =
        serde_json::from_str(include_str!("fixtures/live/search.json")).expect("search fixture");
    let list: Vec<AppOverview> =
        serde_json::from_str(include_str!("fixtures/live/list.json")).expect("list fixture");
    let reviews: Vec<Review> =
        serde_json::from_str(include_str!("fixtures/live/reviews.json")).expect("reviews fixture");
    let suggestions: Vec<String> =
        serde_json::from_str(include_str!("fixtures/live/suggestions.json"))
            .expect("suggestions fixture");

    assert_generic_app(&app.overview);
    assert!(!app.screenshot_urls.is_empty());
    for overview in search.iter().chain(&list) {
        assert_generic_app(overview);
    }
    assert!(!reviews.is_empty());
    assert!(
        reviews
            .iter()
            .all(|review| review.id == "example-review-id" && review.user_name == "Example User")
    );
    assert!(
        suggestions
            .iter()
            .all(|suggestion| suggestion == "example suggestion")
    );
}

fn assert_generic_app(app: &AppOverview) {
    assert!(app.app_id.as_str().starts_with("com.example."));
    assert_eq!(app.title, "Example App");
    assert_eq!(
        app.store_url.host_str(),
        Some("play.google.com"),
        "canonical URLs may retain the Google Play host"
    );
    if let Some(icon) = &app.icon_url {
        assert_eq!(icon.host_str(), Some("example.invalid"));
    }
}
