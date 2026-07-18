use super::*;

#[test]
fn parses_review_page_and_reply() {
    let page = parse_reviews(&crate::test_support::fixtures::reviews_rpc()).unwrap();
    assert_eq!(page.items[0].user_name, "Example User");
    assert_eq!(
        page.items[0].developer_reply.as_ref().unwrap().text,
        "Example reply"
    );
    assert_eq!(page.next_page_token.unwrap().expose(), "NEXT_REVIEW_TOKEN");
}
