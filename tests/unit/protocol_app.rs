use super::*;

#[test]
fn parses_complete_sanitized_app_fixture() {
    let app = parse_app(
        &crate::test_support::fixtures::app_html(),
        AppId::new("com.example.app").unwrap(),
    )
    .unwrap();
    assert_eq!(app.overview.title, "Example App");
    assert_eq!(app.description.as_deref(), Some("Line one\r\nLine two"));
    assert_eq!(app.overview.price.as_ref().unwrap().micros, 1_990_000);
    assert_eq!(app.histogram.as_ref().unwrap().five_star, 5);
    assert_eq!(app.screenshot_urls[0].host_str(), Some("example.invalid"));
}

#[test]
fn structural_drift_is_not_reported_as_not_found() {
    let error = parse_app("<html></html>", AppId::new("com.example.app").unwrap()).unwrap_err();
    assert_eq!(error.kind(), crate::ErrorKind::UnexpectedResponse);
}
