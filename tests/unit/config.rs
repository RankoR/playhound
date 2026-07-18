use super::*;

#[test]
fn proxy_schemes_are_validated() {
    assert!(Proxy::all("http://proxy.example:8080").is_ok());
    assert!(Proxy::http("socks5h://proxy.example:1080").is_ok());
    assert!(Proxy::https("ftp://proxy.example/file").is_err());
    assert!(Proxy::all("not a URL").is_err());
}

#[test]
fn proxy_debug_output_redacts_user_info() {
    let proxy = Proxy::all("http://user:password@proxy.example:8080").unwrap();
    let debug = format!("{proxy:?}");
    assert!(!debug.contains("user"));
    assert!(!debug.contains("password"));
    assert!(debug.contains("proxy.example"));
}

#[test]
fn retries_are_off_by_default_and_configurable() {
    let none = RetryPolicy::default();
    assert_eq!(none.max_retries, 0);
    let policy = RetryPolicy::exponential(3)
        .base_delay(Duration::from_millis(10))
        .max_delay(Duration::from_secs(1))
        .honor_retry_after(false);
    assert_eq!(policy.max_retries, 3);
    assert!(!policy.honor_retry_after);
}
