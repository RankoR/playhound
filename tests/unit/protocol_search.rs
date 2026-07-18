use super::*;
use crate::test_support::fixtures::{search_html, search_item, search_rpc};

#[test]
fn parses_initial_and_continuation_search_pages() {
    let html = search_html(
        vec![search_item(Some("com.example.one"), "First")],
        Some("NEXT"),
    );
    let (items, token) = parse_initial_search(&html).unwrap();
    assert_eq!(items[0].title, "First");
    assert_eq!(token.unwrap().expose(), "NEXT");

    let body = search_rpc(vec![search_item(Some("com.example.two"), "Second")], None);
    let (items, token) = parse_search_page(&body).unwrap();
    assert_eq!(items[0].app_id.as_str(), "com.example.two");
    assert!(token.is_none());
}

#[test]
fn parses_current_store_search_cards_without_treating_tracking_data_as_a_token() {
    let html = crate::test_support::fixtures::current_search_html(Some("CURRENT_TOKEN"));
    let (items, token) = parse_initial_search(&html).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].app_id.as_str(), "com.example.current");
    assert_eq!(items[0].title, "Current Search App");
    assert!(token.is_none());
}

#[test]
fn discovers_current_results_after_a_featured_section() {
    let html = crate::test_support::fixtures::current_search_html_after_featured_section();
    let (items, token) = parse_initial_search(&html).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].app_id.as_str(), "com.example.current");
    assert!(token.is_none());
}

#[test]
fn skips_item_without_identity() {
    let html = search_html(vec![search_item(None, "Missing")], None);
    assert!(parse_initial_search(&html).unwrap().0.is_empty());
}
