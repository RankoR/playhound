//! Repository invariants that keep tests offline, separate, and privacy-safe.

use std::{fs, path::Path};

#[test]
fn production_source_contains_no_inline_test_functions() {
    visit_rs_files(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(),
        &mut |path, text| {
            assert!(
                !text.contains("#[test]") && !text.contains("#[tokio::test]"),
                "test function must be moved out of production source: {}",
                path.display()
            );
        },
    );
}

#[test]
fn committed_fixture_sources_use_only_generic_identities() {
    let mut fixtures = include_str!("support/fixtures.rs").to_owned();
    for fixture in [
        include_str!("fixtures/live/app.json"),
        include_str!("fixtures/live/search.json"),
        include_str!("fixtures/live/list.json"),
        include_str!("fixtures/live/reviews.json"),
        include_str!("fixtures/live/suggestions.json"),
    ] {
        fixtures.push_str(fixture);
    }
    for forbidden in [
        "googleusercontent.com",
        "ggpht.com",
        "@gmail.com",
        "com.google.",
        "com.facebook.",
        "com.spotify.",
    ] {
        assert!(
            !fixtures.contains(forbidden),
            "fixture contains forbidden value: {forbidden}"
        );
    }
    assert!(fixtures.contains("com.example."));
    assert!(fixtures.contains("example.invalid"));
}

fn visit_rs_files(directory: &Path, visitor: &mut impl FnMut(&Path, &str)) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            visit_rs_files(&path, visitor);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            let text = fs::read_to_string(&path).unwrap();
            visitor(&path, &text);
        }
    }
}
