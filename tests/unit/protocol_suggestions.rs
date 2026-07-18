use super::*;

#[test]
fn parses_suggestions_fixture() {
    assert_eq!(
        parse_suggestions(&crate::test_support::fixtures::suggestions_rpc()).unwrap(),
        ["example app", "example game"]
    );
}
