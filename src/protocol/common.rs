use serde_json::Value;
use url::Url;

use crate::{AppId, AppOverview, Money, parser};

pub(crate) fn store_url(app_id: &AppId) -> Url {
    Url::parse(&format!(
        "https://play.google.com/store/apps/details?id={app_id}"
    ))
    .expect("validated app ID always creates a valid URL")
}

pub(crate) fn developer_id(raw: Option<String>) -> Option<String> {
    raw.map(|value| {
        value
            .split_once("id=")
            .map_or(value.clone(), |(_, id)| id.to_owned())
    })
}

pub(crate) fn money(
    root: &Value,
    micros_path: &[usize],
    currency_path: &[usize],
    formatted_path: &[usize],
) -> Option<Money> {
    let micros = parser::signed(root, micros_path)?;
    Some(Money {
        micros,
        currency: parser::text(root, currency_path),
        formatted: parser::text(root, formatted_path),
    })
}

pub(crate) fn search_overview(item: &Value) -> Option<AppOverview> {
    let app_id = parser::text(item, &[12, 0]).and_then(|id| AppId::new(id).ok())?;
    let title = parser::text(item, &[2])?;
    let price = money(
        item,
        &[7, 0, 3, 2, 1, 0, 0],
        &[7, 0, 3, 2, 1, 0, 1],
        &[7, 0, 3, 2, 1, 0, 2],
    );
    let is_free = price.as_ref().map(Money::is_free);
    Some(AppOverview {
        store_url: store_url(&app_id),
        app_id,
        title,
        icon_url: parser::url(item, &[1, 1, 0, 3, 2]),
        developer: parser::text(item, &[4, 0, 0, 0]),
        developer_id: developer_id(parser::text(item, &[4, 0, 0, 1, 4, 2])),
        score: parser::float(item, &[6, 0, 2, 1, 1]),
        score_text: parser::text(item, &[6, 0, 2, 1, 0]),
        price,
        is_free,
        summary: parser::text(item, &[4, 1, 1, 1, 1]),
    })
}

pub(crate) fn card_overview(raw: &Value) -> Option<AppOverview> {
    let item = parser::at(raw, &[0])?;
    let app_id = parser::text(item, &[0, 0]).and_then(|value| AppId::new(value).ok())?;
    let title = parser::text(item, &[3])?;
    let price = money(item, &[8, 1, 0, 0], &[8, 1, 0, 1], &[8, 1, 0, 2]);
    let is_free = price.as_ref().map(Money::is_free);
    Some(AppOverview {
        store_url: store_url(&app_id),
        app_id,
        title,
        icon_url: parser::url(item, &[1, 3, 2]),
        developer: parser::text(item, &[14]),
        developer_id: developer_id(parser::text(item, &[14])),
        score: parser::float(item, &[4, 1]),
        score_text: parser::text(item, &[4, 0]),
        price,
        is_free,
        summary: parser::text(item, &[13, 1]),
    })
}
