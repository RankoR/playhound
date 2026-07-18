use std::{num::NonZeroU32, process::ExitCode, time::Duration};

use clap::{Args, Parser, Subcommand};
use playhound::{
    AgeRange, Category, Client, Collection, Error, ErrorKind, ListQuery, Locale, PriceFilter,
    Proxy, RetryPolicy, ReviewQuery, ReviewSort, SearchQuery,
};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Parser)]
#[command(
    name = "playhound",
    version,
    about = "Inspect public Google Play metadata"
)]
struct Cli {
    #[command(flatten)]
    global: Global,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Args)]
struct Global {
    #[arg(long, env = "PLAYHOUND_LANG", default_value = "en", global = true)]
    lang: String,
    #[arg(long, env = "PLAYHOUND_COUNTRY", default_value = "us", global = true)]
    country: String,
    #[arg(long, env = "PLAYHOUND_PROXY", global = true)]
    proxy: Option<String>,
    #[arg(long, env = "PLAYHOUND_HTTP_PROXY", global = true)]
    http_proxy: Option<String>,
    #[arg(long, env = "PLAYHOUND_HTTPS_PROXY", global = true)]
    https_proxy: Option<String>,
    #[arg(long, env = "PLAYHOUND_NO_SYSTEM_PROXY", global = true)]
    no_system_proxy: bool,
    #[arg(long, env = "PLAYHOUND_REQUESTS_PER_SECOND", global = true)]
    requests_per_second: Option<NonZeroU32>,
    #[arg(long, env = "PLAYHOUND_TIMEOUT", default_value_t = 30, global = true)]
    timeout: u64,
    #[arg(
        long,
        env = "PLAYHOUND_CONNECT_TIMEOUT",
        default_value_t = 10,
        global = true
    )]
    connect_timeout: u64,
    #[arg(
        long,
        env = "PLAYHOUND_MAX_RESPONSE_BYTES",
        default_value_t = 33_554_432,
        global = true
    )]
    max_response_bytes: usize,
    #[arg(long, env = "PLAYHOUND_RETRIES", default_value_t = 0, global = true)]
    retries: u32,
    #[arg(long, env = "PLAYHOUND_INSECURE", global = true)]
    insecure: bool,
    #[arg(long, global = true)]
    json: bool,
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Fetch complete metadata for an application.
    App { app_id: String },
    /// Search Google Play applications.
    Search {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long, default_value = "all", value_parser = parse_price)]
        price: PriceFilter,
    },
    /// Fetch an application collection.
    List {
        #[arg(long, default_value = "topselling_free")]
        collection: Collection,
        #[arg(long, default_value = "APPLICATION")]
        category: Category,
        #[arg(long, value_parser = parse_age)]
        age: Option<AgeRange>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Fetch one or more review pages.
    Reviews {
        app_id: String,
        #[arg(long, default_value = "newest", value_parser = parse_review_sort)]
        sort: ReviewSort,
        #[arg(long, default_value_t = 100)]
        page_size: usize,
        #[arg(long)]
        page_token: Option<String>,
        #[arg(long, default_value_t = 1)]
        pages: usize,
    },
    /// Fetch search suggestions.
    Suggest { query: String },
    /// Exercise every supported operation and report a summary.
    CheckAll {
        #[arg(long)]
        app_id: String,
        #[arg(long)]
        query: String,
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },
}

pub(crate) async fn main_entry() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.global.verbose);
    match build_client(&cli.global) {
        Ok(client) => match execute(&client, &cli.command, cli.global.json).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                print_error(&error, cli.global.json);
                exit_code(&error)
            }
        },
        Err(error) => {
            print_error(&error, cli.global.json);
            exit_code(&error)
        }
    }
}

fn build_client(global: &Global) -> playhound::Result<Client> {
    let locale = Locale::new(&global.lang, &global.country)?;
    let mut builder = Client::builder()
        .default_locale(locale)
        .use_system_proxy(!global.no_system_proxy)
        .request_timeout(Duration::from_secs(global.timeout))
        .connect_timeout(Duration::from_secs(global.connect_timeout))
        .max_response_bytes(global.max_response_bytes)
        .retry_policy(RetryPolicy::exponential(global.retries))
        .danger_accept_invalid_certs(global.insecure);
    if let Some(rate) = global.requests_per_second {
        builder = builder.requests_per_second(rate);
    }
    if let Some(proxy) = &global.proxy {
        builder = builder.proxy(Proxy::all(proxy)?);
    }
    if let Some(proxy) = &global.http_proxy {
        builder = builder.proxy(Proxy::http(proxy)?);
    }
    if let Some(proxy) = &global.https_proxy {
        builder = builder.proxy(Proxy::https(proxy)?);
    }
    builder.build()
}

async fn execute(client: &Client, command: &Command, json_output: bool) -> playhound::Result<()> {
    match command {
        Command::App { app_id } => {
            output(&client.app(app_id.as_str()).await?, json_output, app_human)
        }
        Command::Search {
            query,
            limit,
            price,
        } => output_slice(
            &client
                .search(SearchQuery::new(query).limit(*limit).price(*price))
                .await?,
            json_output,
            apps_human,
        ),
        Command::List {
            collection,
            category,
            age,
            limit,
        } => {
            let mut request = ListQuery::new(collection.clone(), category.clone()).limit(*limit);
            if let Some(age) = age {
                request = request.age(*age);
            }
            output_slice(&client.list(request).await?, json_output, apps_human)
        }
        Command::Reviews {
            app_id,
            sort,
            page_size,
            page_token,
            pages,
        } => {
            if *pages == 0 {
                return Err(Error::InvalidInput {
                    field: "pages",
                    message: "must be nonzero".into(),
                });
            }
            let mut token = page_token
                .as_ref()
                .map(playhound::PageToken::new)
                .transpose()?;
            let mut all = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for _ in 0..*pages {
                let mut request = ReviewQuery::new(app_id).sort(*sort).page_size(*page_size);
                if let Some(value) = token.take() {
                    request = request.page_token(value);
                }
                let page = client.reviews(request).await?;
                all.extend(page.items);
                let Some(next) = page.next_page_token else {
                    break;
                };
                if !seen.insert(next.expose().to_owned()) {
                    break;
                }
                token = Some(next);
            }
            output_slice(&all, json_output, reviews_human)
        }
        Command::Suggest { query } => output_slice(
            &client.suggestions(query.as_str()).await?,
            json_output,
            suggestions_human,
        ),
        Command::CheckAll {
            app_id,
            query,
            limit,
        } => check_all(client, app_id, query, *limit, json_output).await,
    }
}

async fn check_all(
    client: &Client,
    app_id: &str,
    query: &str,
    limit: usize,
    json_output: bool,
) -> playhound::Result<()> {
    let mut report = serde_json::Map::new();
    let mut failed = false;
    macro_rules! check {
        ($name:literal, $future:expr) => {
            match $future.await {
                Ok(value) => { report.insert($name.into(), json!({"ok": true, "value": value})); }
                Err(error) => { failed = true; report.insert($name.into(), json!({"ok": false, "error": error.to_string(), "kind": format!("{:?}", error.kind())})); }
            }
        };
    }
    check!("app", client.app(app_id));
    check!(
        "search",
        client.search(SearchQuery::new(query).limit(limit))
    );
    check!("list", client.list(ListQuery::default().limit(limit)));
    check!(
        "reviews",
        client.reviews(ReviewQuery::new(app_id).page_size(limit))
    );
    check!("suggestions", client.suggestions(query));
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&Value::Object(report)).expect("JSON serialization")
        );
    } else {
        for (name, result) in &report {
            let status = if result["ok"].as_bool() == Some(true) {
                "OK"
            } else {
                "FAILED"
            };
            println!("{name:12} {status}");
            if status == "FAILED" {
                println!(
                    "             {}",
                    result["error"].as_str().unwrap_or("unknown error")
                );
            }
        }
    }
    if failed {
        Err(Error::UnexpectedResponse {
            operation: "check-all",
            message: "one or more checks failed".into(),
        })
    } else {
        Ok(())
    }
}

fn output<T: Serialize>(value: &T, json_output: bool, human: fn(&T)) -> playhound::Result<()> {
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(value).map_err(|error| Error::Parse {
                operation: "CLI JSON",
                message: error.to_string()
            })?
        );
    } else {
        human(value);
    }
    Ok(())
}

fn output_slice<T: Serialize>(
    value: &[T],
    json_output: bool,
    human: fn(&[T]),
) -> playhound::Result<()> {
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(value).map_err(|error| Error::Parse {
                operation: "CLI JSON",
                message: error.to_string()
            })?
        );
    } else {
        human(value);
    }
    Ok(())
}

fn app_human(app: &playhound::AppDetails) {
    println!("{} ({})", app.overview.title, app.overview.app_id);
    if let Some(developer) = &app.overview.developer {
        println!("Developer: {developer}");
    }
    if let Some(score) = app.overview.score {
        println!("Score:     {score:.2}");
    }
    if let Some(installs) = &app.installs_text {
        println!("Installs:  {installs}");
    }
    if let Some(version) = &app.version {
        println!("Version:   {version}");
    }
}

fn apps_human(apps: &[playhound::AppOverview]) {
    for (index, app) in apps.iter().enumerate() {
        println!("{:>3}. {} ({})", index + 1, app.title, app.app_id);
        if let Some(developer) = &app.developer {
            println!("     {developer}");
        }
    }
}

fn reviews_human(reviews: &[playhound::Review]) {
    for review in reviews {
        println!(
            "[{}/5] {}: {}",
            review.score,
            review.user_name,
            review.text.as_deref().unwrap_or("")
        );
    }
}

fn suggestions_human(values: &[String]) {
    for value in values {
        println!("{value}");
    }
}

fn print_error(error: &Error, json_output: bool) {
    if json_output {
        eprintln!(
            "{}",
            json!({"ok": false, "error": error.to_string(), "kind": format!("{:?}", error.kind())})
        );
    } else {
        eprintln!("error: {error}");
    }
}

fn exit_code(error: &Error) -> ExitCode {
    ExitCode::from(match error.kind() {
        ErrorKind::InvalidInput | ErrorKind::Configuration => 2,
        ErrorKind::NotFound => 3,
        ErrorKind::RateLimited => 4,
        ErrorKind::HttpStatus | ErrorKind::Transport | ErrorKind::ResponseTooLarge => 5,
        ErrorKind::UnexpectedResponse | ErrorKind::Parse => 6,
        _ => 1,
    })
}

fn init_tracing(verbosity: u8) {
    let fallback = match verbosity {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| fallback.into()),
        )
        .with_writer(std::io::stderr)
        .try_init();
}

fn parse_price(value: &str) -> Result<PriceFilter, String> {
    match value.to_ascii_lowercase().as_str() {
        "all" => Ok(PriceFilter::All),
        "free" => Ok(PriceFilter::Free),
        "paid" => Ok(PriceFilter::Paid),
        _ => Err("expected all, free, or paid".into()),
    }
}
fn parse_review_sort(value: &str) -> Result<ReviewSort, String> {
    match value.to_ascii_lowercase().as_str() {
        "newest" => Ok(ReviewSort::Newest),
        "rating" => Ok(ReviewSort::Rating),
        "helpfulness" => Ok(ReviewSort::Helpfulness),
        _ => Err("expected newest, rating, or helpfulness".into()),
    }
}
fn parse_age(value: &str) -> Result<AgeRange, String> {
    match value.to_ascii_lowercase().as_str() {
        "five-and-under" | "5" => Ok(AgeRange::FiveAndUnder),
        "six-to-eight" | "6-8" => Ok(AgeRange::SixToEight),
        "nine-and-up" | "9+" => Ok(AgeRange::NineAndUp),
        _ => Err("expected five-and-under, six-to-eight, or nine-and-up".into()),
    }
}

#[cfg(test)]
#[path = "../tests/unit/cli.rs"]
mod tests;
