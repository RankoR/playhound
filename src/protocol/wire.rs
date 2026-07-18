use reqwest::Method;
use serde_json::{Value, json};

use crate::{
    AppId, Error, ListQuery, Locale, Result, ReviewQuery, SearchQuery, SuggestionQuery,
    transport::HttpRequest,
};

const RPC_PATH: &str = "/_/PlayStoreUi/data/batchexecute";

fn locale_query(locale: &Locale) -> Vec<(String, String)> {
    vec![
        ("hl".into(), locale.language.as_str().into()),
        ("gl".into(), locale.country.as_str().into()),
    ]
}

pub(crate) fn app_request(app_id: &AppId, locale: &Locale) -> HttpRequest {
    let mut query = locale_query(locale);
    query.push(("id".into(), app_id.to_string()));
    HttpRequest {
        method: Method::GET,
        path: "/store/apps/details".into(),
        query,
        form: None,
    }
}

pub(crate) fn search_request(query_value: &SearchQuery, locale: &Locale) -> HttpRequest {
    let price = match query_value.price {
        crate::PriceFilter::All => 0,
        crate::PriceFilter::Free => 1,
        crate::PriceFilter::Paid => 2,
    };
    let mut query = locale_query(locale);
    query.extend([
        ("q".into(), query_value.term.clone()),
        ("c".into(), "apps".into()),
        ("price".into(), price.to_string()),
    ]);
    HttpRequest {
        method: Method::GET,
        path: "/store/search".into(),
        query,
        form: None,
    }
}

pub(crate) fn search_page_request(token: &str, locale: &Locale) -> HttpRequest {
    let inner = json!([[
        null,
        [
            [10, [10, 100]],
            true,
            null,
            [
                96, 27, 4, 8, 57, 30, 110, 79, 11, 16, 49, 1, 3, 9, 12, 104, 55, 56, 51, 10, 34, 77
            ]
        ],
        null,
        token
    ]]);
    rpc_request(
        "qnKhOb",
        inner,
        locale,
        vec![
            ("f.sid".into(), "-697906427155521722".into()),
            ("bl".into(), "boq_playuiserver_20190903.08_p0".into()),
            ("authuser".into(), String::new()),
            ("soc-app".into(), "121".into()),
            ("soc-platform".into(), "1".into()),
            ("soc-device".into(), "1".into()),
            ("_reqid".into(), "1065213".into()),
        ],
    )
}

pub(crate) fn list_request(request: &ListQuery, locale: &Locale) -> Result<HttpRequest> {
    let mut outer: Value = serde_json::from_str(include_str!(
        "../../data/list_request.json.template"
    ))
    .map_err(|error| Error::Configuration {
        message: format!("invalid embedded list template: {error}"),
    })?;
    let inner = outer
        .pointer("/0/0/1")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Configuration {
            message: "embedded list template has no inner request".into(),
        })?;
    let inner = inner
        .replace("{num}", &request.limit.to_string())
        .replace(
            "\"{collection}\"",
            &serde_json::to_string(request.collection.as_str()).expect("string serialization"),
        )
        .replace(
            "\"{category}\"",
            &serde_json::to_string(request.category.as_str()).expect("string serialization"),
        );
    let structured: Value = serde_json::from_str(&inner).map_err(|error| Error::Configuration {
        message: format!("invalid embedded list request: {error}"),
    })?;
    *outer
        .pointer_mut("/0/0/1")
        .expect("validated template path") =
        Value::String(serde_json::to_string(&structured).expect("value serialization"));
    let mut query = locale_query(locale);
    query.extend([
        ("rpcids".into(), "vyAe2".into()),
        ("source-path".into(), "/store/apps".into()),
        ("f.sid".into(), "-4178618388443751758".into()),
        ("bl".into(), "boq_playuiserver_20220612.08_p0".into()),
        ("authuser".into(), "0".into()),
        ("soc-app".into(), "121".into()),
        ("soc-platform".into(), "1".into()),
        ("soc-device".into(), "1".into()),
        ("_reqid".into(), "82003".into()),
        ("rt".into(), "c".into()),
    ]);
    if let Some(age) = request.age {
        query.push(("age".into(), age.as_str().into()));
    }
    Ok(HttpRequest {
        method: Method::POST,
        path: RPC_PATH.into(),
        query,
        form: Some(vec![(
            "f.req".into(),
            serde_json::to_string(&outer).expect("value serialization"),
        )]),
    })
}

pub(crate) fn review_request(
    request: &ReviewQuery,
    app_id: &AppId,
    locale: &Locale,
) -> HttpRequest {
    let inner = json!([
        null,
        null,
        [
            2,
            request.sort.wire_value(),
            [
                request.page_size,
                null,
                request.page_token.as_ref().map(crate::PageToken::expose)
            ],
            null,
            []
        ],
        [app_id.as_str(), 7]
    ]);
    rpc_request("UsvDTd", inner, locale, Vec::new())
}

pub(crate) fn suggestion_request(request: &SuggestionQuery, locale: &Locale) -> HttpRequest {
    let inner = json!([[null, [request.term], [10], [2], 4]]);
    rpc_request(
        "IJ4APc",
        inner,
        locale,
        vec![
            ("bl".into(), "boq_playuiserver_20190903.08_p0".into()),
            ("authuser".into(), "0".into()),
            ("soc-app".into(), "121".into()),
            ("soc-platform".into(), "1".into()),
            ("soc-device".into(), "1".into()),
            ("rt".into(), "c".into()),
        ],
    )
}

fn rpc_request(
    rpc_id: &str,
    inner: Value,
    locale: &Locale,
    extra_query: Vec<(String, String)>,
) -> HttpRequest {
    let inner = serde_json::to_string(&inner).expect("value serialization");
    let outer = json!([[[rpc_id, inner, null, "generic"]]]);
    let mut query = locale_query(locale);
    query.push(("rpcids".into(), rpc_id.into()));
    query.extend(extra_query);
    HttpRequest {
        method: Method::POST,
        path: RPC_PATH.into(),
        query,
        form: Some(vec![(
            "f.req".into(),
            serde_json::to_string(&outer).expect("value serialization"),
        )]),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/protocol_wire.rs"]
mod tests;
