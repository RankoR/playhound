use std::collections::HashMap;

use jiff::Timestamp;
use scraper::{Html, Selector};
use serde_json::Value;
use url::Url;

use crate::{Error, Result};

pub(crate) fn parse_html_data(html: &str) -> Result<HashMap<String, Value>> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("script")
        .map_err(|_| Error::unexpected("HTML", "internal script selector is invalid"))?;
    let mut output = HashMap::new();

    for script in document.select(&selector) {
        let text = script.text().collect::<String>();
        if !text.contains("AF_initDataCallback") {
            continue;
        }
        let Some(key) = extract_ds_key(&text) else {
            continue;
        };
        let Some(data_start) = text.find("data:").map(|index| index + "data:".len()) else {
            continue;
        };
        let input = text[data_start..].trim_start();
        let mut values = serde_json::Deserializer::from_str(input).into_iter::<Value>();
        if let Some(Ok(value)) = values.next() {
            output.insert(key, value);
        }
    }

    Ok(output)
}

fn extract_ds_key(script: &str) -> Option<String> {
    let key_start = script.find("key:")? + 4;
    let input = script[key_start..].trim_start();
    let quote = input.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let rest = &input[quote.len_utf8()..];
    let end = rest.find(quote)?;
    let key = &rest[..end];
    key.starts_with("ds:").then(|| key.to_owned())
}

pub(crate) fn parse_rpc_response(input: &str, rpc_id: &str) -> Result<Value> {
    let input = input.strip_prefix(")]}'").unwrap_or(input).trim();
    if let Ok(value) = serde_json::from_str::<Value>(input)
        && let Some(inner) = find_rpc_inner(&value, rpc_id)?
    {
        return Ok(inner);
    }
    for line in input
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("[["))
    {
        if let Ok(value) = serde_json::from_str::<Value>(line)
            && let Some(inner) = find_rpc_inner(&value, rpc_id)?
        {
            return Ok(inner);
        }
    }
    Err(Error::unexpected(
        "RPC",
        format!("no valid {rpc_id} envelope"),
    ))
}

fn find_rpc_inner(value: &Value, rpc_id: &str) -> Result<Option<Value>> {
    let Some(rows) = value.as_array() else {
        return Ok(None);
    };
    for row in rows {
        if let Some(columns) = row.as_array() {
            if columns.first().and_then(Value::as_str) == Some("wrb.fr")
                && columns.get(1).and_then(Value::as_str) == Some(rpc_id)
            {
                let inner = columns
                    .get(2)
                    .and_then(Value::as_str)
                    .ok_or_else(|| Error::unexpected("RPC", "matched envelope has no JSON body"))?;
                return serde_json::from_str(inner)
                    .map(Some)
                    .map_err(|error| Error::Parse {
                        operation: "RPC",
                        message: error.to_string(),
                    });
            }
            if let Some(found) = find_rpc_inner(row, rpc_id)? {
                return Ok(Some(found));
            }
        }
    }
    Ok(None)
}

pub(crate) fn at<'a>(value: &'a Value, path: &[usize]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, index| current.as_array()?.get(*index))
}

pub(crate) fn text(value: &Value, path: &[usize]) -> Option<String> {
    at(value, path)?.as_str().map(ToOwned::to_owned)
}

pub(crate) fn unsigned(value: &Value, path: &[usize]) -> Option<u64> {
    let value = at(value, path)?;
    value
        .as_u64()
        .or_else(|| value.as_i64().and_then(|number| u64::try_from(number).ok()))
        .or_else(|| {
            value.as_str().and_then(|string| {
                let digits: String = string.chars().filter(char::is_ascii_digit).collect();
                (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
            })
        })
}

pub(crate) fn signed(value: &Value, path: &[usize]) -> Option<i64> {
    let value = at(value, path)?;
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|number| i64::try_from(number).ok()))
}

pub(crate) fn float(value: &Value, path: &[usize]) -> Option<f64> {
    let value = at(value, path)?;
    value
        .as_f64()
        .or_else(|| value.as_str()?.replace(',', ".").parse().ok())
}

pub(crate) fn boolean(value: &Value, path: &[usize]) -> Option<bool> {
    let value = at(value, path)?;
    value
        .as_bool()
        .or_else(|| value.as_i64().map(|number| number != 0))
}

pub(crate) fn url(value: &Value, path: &[usize]) -> Option<Url> {
    text(value, path).and_then(|raw| Url::parse(&raw).ok())
}

pub(crate) fn timestamp(value: &Value, path: &[usize]) -> Option<Timestamp> {
    signed(value, path).and_then(|seconds| Timestamp::from_second(seconds).ok())
}

#[cfg(test)]
#[path = "../tests/unit/parser.rs"]
mod tests;
