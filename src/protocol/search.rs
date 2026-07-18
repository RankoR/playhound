use crate::{
    AppOverview, Error, PageToken, Result, parser,
    protocol::common::{card_overview, search_overview},
};

pub(crate) fn parse_initial_search(html: &str) -> Result<(Vec<AppOverview>, Option<PageToken>)> {
    let data = parser::parse_html_data(html)?;
    if let Some(ds4) = data.get("ds:4") {
        if let Some(items) = current_result_cards(ds4) {
            let results = extract_cards(items);
            // Current storefront responses carry an impression-tracking value at
            // ds:4[1][0], not a search continuation token. The server currently
            // caps storefront search results at roughly 30 items.
            return Ok((results, None));
        }
    }
    let ds1 = data
        .get("ds:1")
        .ok_or_else(|| Error::unexpected("search", "missing recognized search-result payload"))?;
    let sections = parser::at(ds1, &[0, 1, 0, 0])
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| Error::unexpected("search", "invalid search section root"))?;
    let items = sections
        .first()
        .and_then(serde_json::Value::as_array)
        .map_or(&[] as &[serde_json::Value], Vec::as_slice);
    let results = extract(items, "search");
    let token = sections
        .iter()
        .find_map(|section| parser::text(section, &[1]))
        .and_then(|value| PageToken::new(value).ok());
    Ok((results, token))
}

fn current_result_cards(root: &serde_json::Value) -> Option<&[serde_json::Value]> {
    parser::at(root, &[0, 1])
        .and_then(serde_json::Value::as_array)?
        .iter()
        .find_map(|section| {
            parser::at(section, &[22, 0])
                .and_then(serde_json::Value::as_array)
                .map(Vec::as_slice)
        })
}

fn extract_cards(items: &[serde_json::Value]) -> Vec<AppOverview> {
    items
        .iter()
        .filter_map(|item| {
            let parsed = card_overview(item);
            if parsed.is_none() {
                tracing::warn!(
                    operation = "search",
                    "skipping search card without required identity"
                );
            }
            parsed
        })
        .collect()
}

pub(crate) fn parse_search_page(input: &str) -> Result<(Vec<AppOverview>, Option<PageToken>)> {
    let data = parser::parse_rpc_response(input, "qnKhOb")?;
    let items = parser::at(&data, &[0, 0, 0])
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| Error::unexpected("search", "invalid continuation items"))?;
    let token = parser::text(&data, &[0, 0, 7, 1]).and_then(|value| PageToken::new(value).ok());
    Ok((extract(items, "search continuation"), token))
}

fn extract(items: &[serde_json::Value], operation: &'static str) -> Vec<AppOverview> {
    items
        .iter()
        .filter_map(|item| {
            let parsed = search_overview(item);
            if parsed.is_none() {
                tracing::warn!(operation, "skipping search item without required identity");
            }
            parsed
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/protocol_search.rs"]
mod tests;
