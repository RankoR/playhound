use super::*;

#[test]
fn parses_list_fixture() {
    let apps = parse_list(&crate::test_support::fixtures::list_rpc()).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].app_id.as_str(), "com.example.listed");
    assert_eq!(apps[0].price.as_ref().unwrap().micros, 0);
}
