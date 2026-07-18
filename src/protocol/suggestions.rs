use crate::{Error, Result, parser};

pub(crate) fn parse_suggestions(input: &str) -> Result<Vec<String>> {
    let data = parser::parse_rpc_response(input, "IJ4APc")?;
    let items = parser::at(&data, &[0, 0])
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| Error::unexpected("suggestions", "invalid suggestion-list root"))?;
    Ok(items
        .iter()
        .filter_map(|item| parser::text(item, &[0]))
        .collect())
}

#[cfg(test)]
#[path = "../../tests/unit/protocol_suggestions.rs"]
mod tests;
