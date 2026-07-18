use super::*;

#[test]
fn request_builders_encode_locale_and_inputs() {
    let locale = Locale::new("pt", "br").unwrap();
    let search = SearchQuery::new("example").price(crate::PriceFilter::Paid);
    let wire = search_request(&search, &locale);
    assert_eq!(wire.path, "/store/search");
    assert!(wire.query.contains(&("c".into(), "apps".into())));
    assert!(wire.query.contains(&("hl".into(), "pt".into())));
    assert!(wire.query.contains(&("gl".into(), "br".into())));
    assert!(wire.query.contains(&("price".into(), "2".into())));

    let list = list_request(&ListQuery::default().limit(12), &locale).unwrap();
    let form = &list.form.unwrap()[0].1;
    assert!(form.contains("topselling_free"));
    assert!(form.contains("APPLICATION"));
}

#[test]
fn proxy_independent_review_payload_contains_token() {
    let locale = Locale::default();
    let token = crate::PageToken::new("TOKEN").unwrap();
    let request = ReviewQuery::new("com.example.app").page_token(token);
    let wire = review_request(&request, &AppId::new("com.example.app").unwrap(), &locale);
    assert!(wire.form.unwrap()[0].1.contains("TOKEN"));
}
