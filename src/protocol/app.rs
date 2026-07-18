use crate::{
    AppDetails, AppId, AppOverview, Error, Money, RatingHistogram, Result, parser,
    protocol::common::{developer_id, money, store_url},
};

pub(crate) fn parse_app(html: &str, requested_id: AppId) -> Result<AppDetails> {
    let data = parser::parse_html_data(html)?;
    let ds5 = data.get("ds:5").ok_or_else(|| {
        if html.contains("not found") || html.contains("We're sorry") {
            Error::AppNotFound {
                app_id: requested_id.to_string(),
            }
        } else {
            Error::unexpected("app", "missing ds:5 application payload")
        }
    })?;
    let root =
        parser::at(ds5, &[1, 2]).ok_or_else(|| Error::unexpected("app", "invalid ds:5 root"))?;
    let title = parser::text(root, &[0, 0])
        .ok_or_else(|| Error::unexpected("app", "application title is missing"))?;
    let price = money(
        root,
        &[57, 0, 0, 0, 0, 1, 0, 0],
        &[57, 0, 0, 0, 0, 1, 0, 1],
        &[57, 0, 0, 0, 0, 1, 0, 2],
    );
    let is_free = price.as_ref().map(Money::is_free);
    let description_html =
        parser::text(root, &[72, 0, 1]).or_else(|| parser::text(root, &[12, 0, 0, 1]));
    let description = description_html
        .as_ref()
        .map(|value| value.replace("<br>", "\r\n"));

    let overview = AppOverview {
        store_url: store_url(&requested_id),
        app_id: requested_id,
        title,
        icon_url: parser::url(root, &[95, 0, 3, 2]),
        developer: parser::text(root, &[68, 0]),
        developer_id: developer_id(parser::text(root, &[68, 1, 4, 2])),
        score: parser::float(root, &[51, 0, 1]),
        score_text: parser::text(root, &[51, 0, 0]),
        price,
        is_free,
        summary: parser::text(root, &[73, 0, 1]),
    };

    let screenshot_urls = parser::at(root, &[78, 0])
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| parser::url(item, &[3, 2]))
        .collect();

    Ok(AppDetails {
        overview,
        description,
        description_html,
        installs_text: parser::text(root, &[13, 0]),
        min_installs: parser::unsigned(root, &[13, 1]),
        max_installs: parser::unsigned(root, &[13, 2]),
        ratings: parser::unsigned(root, &[51, 2, 1]),
        reviews: parser::unsigned(root, &[51, 3, 1]),
        histogram: histogram(root),
        available: parser::boolean(root, &[18, 0]),
        offers_in_app_purchases: parser::boolean(root, &[19, 0]),
        android_version: parser::text(root, &[140, 1, 1, 0, 0, 1]),
        developer_email: parser::text(root, &[69, 1, 0]),
        developer_website: parser::url(root, &[69, 0, 5, 2]),
        developer_address: parser::text(root, &[69, 2, 0]),
        privacy_policy: parser::url(root, &[99, 0, 5, 2]),
        genre: parser::text(root, &[79, 0, 0, 0]),
        genre_id: parser::text(root, &[79, 0, 0, 2]),
        header_image_url: parser::url(root, &[96, 0, 3, 2]),
        screenshot_urls,
        video_url: parser::url(root, &[100, 0, 0, 3, 2]),
        content_rating: parser::text(root, &[9, 0]),
        released: parser::text(root, &[10, 0]),
        updated: parser::timestamp(root, &[145, 0, 1, 0]),
        version: parser::text(root, &[140, 0, 0, 0]),
        recent_changes: parser::text(root, &[144, 1, 1]),
        comments: Vec::new(),
    })
}

fn histogram(root: &serde_json::Value) -> Option<RatingHistogram> {
    let value = parser::at(root, &[51, 1])?;
    Some(RatingHistogram {
        one_star: parser::unsigned(value, &[1, 1])?,
        two_star: parser::unsigned(value, &[2, 1])?,
        three_star: parser::unsigned(value, &[3, 1])?,
        four_star: parser::unsigned(value, &[4, 1])?,
        five_star: parser::unsigned(value, &[5, 1])?,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/protocol_app.rs"]
mod tests;
