use serde_json::{Value, json};

pub(crate) fn set_path(root: &mut Value, path: &[usize], value: Value) {
    let mut current = root;
    for (position, index) in path.iter().copied().enumerate() {
        if !current.is_array() {
            *current = Value::Array(Vec::new());
        }
        let array = current.as_array_mut().expect("array created above");
        while array.len() <= index {
            array.push(Value::Null);
        }
        if position + 1 == path.len() {
            array[index] = value;
            return;
        }
        if array[index].is_null() {
            array[index] = Value::Array(Vec::new());
        }
        current = &mut array[index];
    }
}

pub(crate) fn app_html() -> String {
    let mut root = json!([]);
    set_path(&mut root, &[0, 0], json!("Example App"));
    set_path(&mut root, &[72, 0, 1], json!("Line one<br>Line two"));
    set_path(&mut root, &[73, 0, 1], json!("Example summary"));
    set_path(&mut root, &[13, 0], json!("1,000+"));
    set_path(&mut root, &[13, 1], json!(1000));
    set_path(&mut root, &[13, 2], json!(5000));
    set_path(&mut root, &[51, 0, 0], json!("4.5"));
    set_path(&mut root, &[51, 0, 1], json!(4.5));
    set_path(
        &mut root,
        &[51, 1],
        json!([null, ["1", 1], ["2", 2], ["3", 3], ["4", 4], ["5", 5]]),
    );
    set_path(&mut root, &[51, 2, 1], json!(1234));
    set_path(&mut root, &[51, 3, 1], json!(321));
    set_path(&mut root, &[57, 0, 0, 0, 0, 1, 0, 0], json!(1_990_000));
    set_path(&mut root, &[57, 0, 0, 0, 0, 1, 0, 1], json!("USD"));
    set_path(&mut root, &[57, 0, 0, 0, 0, 1, 0, 2], json!("$1.99"));
    set_path(&mut root, &[18, 0], json!(1));
    set_path(&mut root, &[19, 0], json!(0));
    set_path(&mut root, &[140, 1, 1, 0, 0, 1], json!("7.0"));
    set_path(&mut root, &[140, 0, 0, 0], json!("1.0.0"));
    set_path(&mut root, &[68, 0], json!("Example Developer"));
    set_path(
        &mut root,
        &[68, 1, 4, 2],
        json!("https://play.google.com/store/apps/dev?id=EXAMPLE_DEV"),
    );
    set_path(&mut root, &[69, 1, 0], json!("developer@example.invalid"));
    set_path(&mut root, &[69, 0, 5, 2], json!("https://example.invalid"));
    set_path(&mut root, &[69, 2, 0], json!("Example address"));
    set_path(
        &mut root,
        &[99, 0, 5, 2],
        json!("https://example.invalid/privacy"),
    );
    set_path(&mut root, &[79, 0, 0, 0], json!("Tools"));
    set_path(&mut root, &[79, 0, 0, 2], json!("TOOLS"));
    set_path(
        &mut root,
        &[95, 0, 3, 2],
        json!("https://example.invalid/icon.png"),
    );
    set_path(
        &mut root,
        &[96, 0, 3, 2],
        json!("https://example.invalid/header.png"),
    );
    set_path(
        &mut root,
        &[78, 0],
        json!([[
            null,
            null,
            null,
            [null, null, "https://example.invalid/shot.png"]
        ]]),
    );
    set_path(
        &mut root,
        &[100, 0, 0, 3, 2],
        json!("https://example.invalid/video.mp4"),
    );
    set_path(&mut root, &[9, 0], json!("Everyone"));
    set_path(&mut root, &[10, 0], json!("January 1, 2020"));
    set_path(&mut root, &[145, 0, 1, 0], json!(1_600_000_000));
    set_path(&mut root, &[144, 1, 1], json!("Example changes"));
    let ds5 = json!([null, [null, null, root]]);
    html_data("ds:5", &ds5)
}

pub(crate) fn search_item(app_id: Option<&str>, title: &str) -> Value {
    let mut item = json!([]);
    set_path(
        &mut item,
        &[1, 1, 0, 3, 2],
        json!("https://example.invalid/icon.png"),
    );
    set_path(&mut item, &[2], json!(title));
    set_path(&mut item, &[4, 0, 0, 0], json!("Example Developer"));
    set_path(
        &mut item,
        &[4, 0, 0, 1, 4, 2],
        json!("https://play.google.com/store/apps/dev?id=EXAMPLE_DEV"),
    );
    set_path(&mut item, &[4, 1, 1, 1, 1], json!("Example summary"));
    set_path(&mut item, &[6, 0, 2, 1, 0], json!("4.5"));
    set_path(&mut item, &[6, 0, 2, 1, 1], json!(4.5));
    set_path(&mut item, &[7, 0, 3, 2, 1, 0, 0], json!(0));
    set_path(&mut item, &[7, 0, 3, 2, 1, 0, 1], json!("USD"));
    set_path(&mut item, &[7, 0, 3, 2, 1, 0, 2], json!("$0.00"));
    if let Some(app_id) = app_id {
        set_path(&mut item, &[12, 0], json!(app_id));
    }
    item
}

pub(crate) fn search_html(items: Vec<Value>, token: Option<&str>) -> String {
    let mut sections = vec![Value::Array(items)];
    if let Some(token) = token {
        sections.push(json!([null, token]));
    }
    html_data("ds:1", &json!([["x", [[sections]]]]))
}

pub(crate) fn current_search_html(token: Option<&str>) -> String {
    current_search_html_in_section(token, 0)
}

pub(crate) fn current_search_html_after_featured_section() -> String {
    current_search_html_in_section(None, 1)
}

fn current_search_html_in_section(token: Option<&str>, section: usize) -> String {
    let mut card = json!([]);
    set_path(&mut card, &[0, 0], json!("com.example.current"));
    set_path(&mut card, &[3], json!("Current Search App"));
    set_path(
        &mut card,
        &[1, 3, 2],
        json!("https://example.invalid/current-icon.png"),
    );
    set_path(&mut card, &[4, 0], json!("4.2"));
    set_path(&mut card, &[4, 1], json!(4.2));
    set_path(&mut card, &[8, 1, 0, 0], json!(0));
    set_path(&mut card, &[8, 1, 0, 1], json!("USD"));
    set_path(&mut card, &[13, 1], json!("Current example summary"));
    set_path(&mut card, &[14], json!("Example Developer"));
    let mut ds4 = json!([]);
    if section > 0 {
        set_path(&mut ds4, &[0, 1, 0, 23, 0], json!("featured section"));
    }
    set_path(&mut ds4, &[0, 1, section, 22, 0], json!([[card]]));
    if let Some(token) = token {
        set_path(&mut ds4, &[1, 0], json!(token));
    }
    html_data("ds:4", &ds4)
}

pub(crate) fn search_rpc(items: Vec<Value>, token: Option<&str>) -> String {
    let token_section = token.map_or(Value::Null, |value| json!([null, value]));
    rpc(
        "qnKhOb",
        json!([[[items, null, null, null, null, null, null, token_section]]]),
    )
}

pub(crate) fn list_rpc() -> String {
    let mut app = json!([]);
    set_path(&mut app, &[0, 0], json!("com.example.listed"));
    set_path(&mut app, &[3], json!("Listed App"));
    set_path(
        &mut app,
        &[1, 3, 2],
        json!("https://example.invalid/list-icon.png"),
    );
    set_path(&mut app, &[4, 0], json!("4.0"));
    set_path(&mut app, &[4, 1], json!(4.0));
    set_path(&mut app, &[8, 1, 0, 0], json!(0));
    set_path(&mut app, &[8, 1, 0, 1], json!("USD"));
    set_path(&mut app, &[13, 1], json!("Listed summary"));
    set_path(&mut app, &[14], json!("Listed Developer"));
    let mut inner = json!([]);
    set_path(&mut inner, &[0, 1, 0, 28, 0], json!([[app]]));
    rpc("vyAe2", inner)
}

pub(crate) fn reviews_rpc() -> String {
    let mut review = json!([]);
    set_path(&mut review, &[0], json!("review-1"));
    set_path(&mut review, &[1, 0], json!("Example User"));
    set_path(
        &mut review,
        &[1, 1, 3, 2],
        json!("https://example.invalid/user.png"),
    );
    set_path(&mut review, &[2], json!(5));
    set_path(&mut review, &[4], json!("Example review"));
    set_path(&mut review, &[5, 0], json!(1_600_000_000));
    set_path(&mut review, &[6], json!(7));
    set_path(&mut review, &[7, 1], json!("Example reply"));
    set_path(&mut review, &[7, 2, 0], json!(1_600_000_001));
    set_path(&mut review, &[10], json!("1.0.0"));
    rpc("UsvDTd", json!([[review], [null, "NEXT_REVIEW_TOKEN"]]))
}

pub(crate) fn suggestions_rpc() -> String {
    rpc("IJ4APc", json!([[[["example app"], ["example game"]]]]))
}

pub(crate) fn rpc(id: &str, inner: Value) -> String {
    let body = serde_json::to_string(&inner).expect("fixture JSON");
    serde_json::to_string(&json!([["wrb.fr", id, body, null]])).expect("fixture envelope")
}

fn html_data(key: &str, value: &Value) -> String {
    format!(
        "<html><script>AF_initDataCallback({{key: '{key}', hash: 'fixture', data:{}, sideChannel: {{}}}});</script></html>",
        serde_json::to_string(value).expect("fixture JSON")
    )
}
