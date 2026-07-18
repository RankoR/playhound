use crate::{DeveloperReply, Error, Page, PageToken, Result, Review, parser};

pub(crate) fn parse_reviews(input: &str) -> Result<Page<Review>> {
    let data = parser::parse_rpc_response(input, "UsvDTd")?;
    let reviews = parser::at(&data, &[0])
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| Error::unexpected("reviews", "invalid review-list root"))?;
    let next_page_token = parser::text(&data, &[1, 1]).and_then(|value| PageToken::new(value).ok());
    let items = reviews
        .iter()
        .filter_map(|raw| {
            let id = parser::text(raw, &[0]);
            let user_name = parser::text(raw, &[1, 0]);
            let score = parser::unsigned(raw, &[2]).and_then(|value| u8::try_from(value).ok());
            let (Some(id), Some(user_name), Some(score)) = (id, user_name, score) else {
                tracing::warn!(
                    operation = "reviews",
                    "skipping review without required identity"
                );
                return None;
            };
            let reply_text = parser::text(raw, &[7, 1]);
            Some(Review {
                id,
                user_name,
                score,
                user_image_url: parser::url(raw, &[1, 1, 3, 2]),
                date: parser::timestamp(raw, &[5, 0]),
                title: None,
                text: parser::text(raw, &[4]),
                developer_reply: reply_text.map(|text| DeveloperReply {
                    text,
                    date: parser::timestamp(raw, &[7, 2, 0]),
                }),
                app_version: parser::text(raw, &[10]),
                thumbs_up: parser::unsigned(raw, &[6]),
            })
        })
        .collect();
    Ok(Page {
        items,
        next_page_token,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/protocol_reviews.rs"]
mod tests;
