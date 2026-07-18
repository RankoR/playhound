//! Black-box checks for the public API surface.

use playhound::{AppId, ErrorKind, Locale, Money, Proxy};

#[test]
fn validates_public_value_types() {
    assert_eq!(AppId::new("").unwrap_err().kind(), ErrorKind::InvalidInput);
    assert_eq!(Locale::new("EN", "US").unwrap().language.as_str(), "en");
    assert!(Locale::new("en", "usa").is_err());
    let units = Money::new(1_990_000, Some("USD".into()), None).as_major_units();
    assert!((units - 1.99).abs() < f64::EPSILON);
}

#[test]
fn proxy_debug_redacts_credentials() {
    let proxy = Proxy::all("http://user:password@proxy.example:8080").unwrap();
    let rendered = format!("{proxy:?}");
    assert!(!rendered.contains("user"));
    assert!(!rendered.contains("password"));
    assert!(rendered.contains("***"));
}
