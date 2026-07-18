use super::*;

fn response(status: u16) -> HttpResponse {
    HttpResponse {
        status,
        retry_after: Some(std::time::Duration::from_secs(7)),
        body: "body".into(),
    }
}

#[test]
fn classifies_success_rate_limits_and_other_statuses() {
    assert_eq!(classify(response(200)).unwrap(), "body");
    let limited = classify(response(429)).unwrap_err();
    assert_eq!(limited.kind(), crate::ErrorKind::RateLimited);
    assert!(matches!(
        limited,
        Error::RateLimited { retry_after: Some(delay) }
            if delay == std::time::Duration::from_secs(7)
    ));
    assert_eq!(
        classify(response(503)).unwrap_err().kind(),
        crate::ErrorKind::RateLimited
    );
    assert_eq!(
        classify(response(404)).unwrap_err().kind(),
        crate::ErrorKind::HttpStatus
    );
    assert_eq!(
        classify(response(599)).unwrap_err().kind(),
        crate::ErrorKind::HttpStatus
    );
}

#[test]
fn parses_only_delta_seconds_retry_after_values() {
    let seconds = reqwest::header::HeaderValue::from_static("42");
    assert_eq!(
        parse_retry_after(Some(&seconds)),
        Some(std::time::Duration::from_secs(42))
    );
    let date = reqwest::header::HeaderValue::from_static("Wed, 21 Oct 2015 07:28:00 GMT");
    assert_eq!(parse_retry_after(Some(&date)), None);
    assert_eq!(parse_retry_after(None), None);
}

#[test]
fn credential_bearing_transport_messages_are_redacted() {
    let message = redact_transport_error("request to http://user:pass@proxy.example failed");
    assert!(!message.contains("user"));
    assert!(!message.contains("pass"));
    assert!(!message.contains("proxy.example"));
}
