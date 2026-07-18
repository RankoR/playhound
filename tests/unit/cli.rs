use super::*;

#[test]
fn parses_all_cli_subcommands_without_network() {
    use clap::Parser;
    for args in [
        vec!["playhound", "app", "com.example.app"],
        vec!["playhound", "search", "example"],
        vec!["playhound", "list"],
        vec!["playhound", "reviews", "com.example.app"],
        vec!["playhound", "suggest", "example"],
        vec![
            "playhound",
            "check-all",
            "--app-id",
            "com.example.app",
            "--query",
            "example",
        ],
    ] {
        assert!(Cli::try_parse_from(args).is_ok());
    }
}
