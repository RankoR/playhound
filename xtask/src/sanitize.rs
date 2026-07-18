use std::str::FromStr;

use serde_json::Value;
use url::Url;

/// Kind of normalized live response being sanitized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureKind {
    /// Application details.
    App,
    /// Search result list.
    Search,
    /// Collection result list.
    List,
    /// Review page.
    Reviews,
    /// Suggestion list.
    Suggestions,
}

impl FromStr for FixtureKind {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "app" => Ok(Self::App),
            "search" => Ok(Self::Search),
            "list" => Ok(Self::List),
            "reviews" => Ok(Self::Reviews),
            "suggestions" => Ok(Self::Suggestions),
            _ => Err("fixture kind must be app, search, list, reviews, or suggestions"),
        }
    }
}

/// Replaces identities, free-form text, URLs, and opaque tokens in normalized JSON.
///
/// # Errors
///
/// Returns an error if the resulting fixture still contains an unexpected URL host
/// or application identifier.
pub fn sanitize(kind: FixtureKind, mut value: Value) -> Result<Value, String> {
    sanitize_value(kind, None, &mut value);
    validate_value(None, &value)?;
    Ok(value)
}

fn sanitize_value(kind: FixtureKind, key: Option<&str>, value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (child_key, child) in object {
                sanitize_value(kind, Some(child_key), child);
            }
        }
        Value::Array(array) => {
            for child in array {
                sanitize_value(kind, key, child);
            }
        }
        Value::String(text) => sanitize_string(kind, key, text),
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn sanitize_string(kind: FixtureKind, key: Option<&str>, text: &mut String) {
    let replacement = match key {
        Some("app_id") => Some("com.example.app"),
        Some("id") if kind == FixtureKind::Reviews => Some("example-review-id"),
        Some("title") if kind == FixtureKind::Reviews => Some("Example Review"),
        Some("title") => Some("Example App"),
        Some("developer") => Some("Example Developer"),
        Some("developer_id") => Some("EXAMPLE_DEVELOPER"),
        Some("user_name") => Some("Example User"),
        Some("description") => Some("Example application description."),
        Some("description_html") => Some("Example application description.<br>Second line."),
        Some("summary") => Some("Example application summary."),
        Some("developer_email") => Some("developer@example.invalid"),
        Some("developer_address") => Some("Example address"),
        Some("recent_changes") => Some("Example release notes."),
        Some("text") if kind == FixtureKind::Reviews => Some("Example review text."),
        Some("app_version") if kind == FixtureKind::Reviews => Some("1.2.3"),
        Some("date") if kind == FixtureKind::Reviews => Some("2026-01-02T03:04:05Z"),
        Some("next_page_token") => Some("EXAMPLE_PAGE_TOKEN"),
        Some("store_url") => Some("https://play.google.com/store/apps/details?id=com.example.app"),
        Some(key) if key.ends_with("_url") || key.ends_with("_urls") => {
            Some("https://example.invalid/image.png")
        }
        None if kind == FixtureKind::Suggestions => Some("example suggestion"),
        _ if Url::parse(text).is_ok() => Some("https://example.invalid/resource"),
        _ => None,
    };
    if let Some(replacement) = replacement {
        replacement.clone_into(text);
    }
}

fn validate_value(key: Option<&str>, value: &Value) -> Result<(), String> {
    match value {
        Value::Object(object) => {
            for (child_key, child) in object {
                validate_value(Some(child_key), child)?;
            }
        }
        Value::Array(array) => {
            for child in array {
                validate_value(key, child)?;
            }
        }
        Value::String(text) => {
            if key == Some("app_id") && !text.starts_with("com.example.") {
                return Err("sanitized fixture contains a non-example application ID".into());
            }
            if let Ok(url) = Url::parse(text) {
                match url.host_str() {
                    Some("example.invalid" | "play.google.com") => {}
                    _ => return Err("sanitized fixture contains an unexpected URL host".into()),
                }
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
    Ok(())
}
