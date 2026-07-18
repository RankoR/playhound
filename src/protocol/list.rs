use crate::{AppOverview, Error, Result, parser, protocol::common::card_overview};

pub(crate) fn parse_list(input: &str) -> Result<Vec<AppOverview>> {
    let data = parser::parse_rpc_response(input, "vyAe2")?;
    let items = parser::at(&data, &[0, 1, 0, 28, 0])
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| Error::unexpected("list", "invalid application-list root"))?;
    Ok(items
        .iter()
        .filter_map(|raw| {
            let parsed = card_overview(raw);
            if parsed.is_none() {
                tracing::warn!(
                    operation = "list",
                    "skipping list item without required identity"
                );
            }
            parsed
        })
        .collect())
}

#[cfg(test)]
#[path = "../../tests/unit/protocol_list.rs"]
mod tests;
