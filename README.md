# PlayHound

PlayHound is an unofficial, typed Google Play metadata scraper for Rust. It
provides:

- an async-first Rust library;
- an optional native blocking client;
- a command-line interface;
- application details, search, collections, reviews, and suggestions;
- HTTP, HTTPS, SOCKS5, and SOCKS5h proxy support;
- localized requests, throttling, bounded retries, and response-size limits;
- Serde-compatible public models and structured errors.

Google Play does not provide a supported public API for this storefront data.
Its HTML and internal RPC response formats can change without notice. Use
conservative request rates, cache responses where appropriate, and comply with
all applicable terms and laws.

## Contents

- [Capabilities](#capabilities)
- [Installation](#installation)
- [Feature flags](#feature-flags)
- [Quick start](#quick-start)
- [Async library API](#async-library-api)
- [Blocking library API](#blocking-library-api)
- [Client configuration](#client-configuration)
- [Localization](#localization)
- [Proxies](#proxies)
- [Retries and rate limiting](#retries-and-rate-limiting)
- [Result models](#result-models)
- [Errors](#errors)
- [CLI installation](#cli-installation)
- [CLI command reference](#cli-command-reference)
- [CLI configuration and environment variables](#cli-configuration-and-environment-variables)
- [Operational notes](#operational-notes)
- [Testing](#testing)
- [MSRV and platforms](#msrv-and-platforms)
- [License](#license)

## Capabilities

| Operation | Rust API | CLI | Result |
| --- | --- | --- | --- |
| Application details | `Client::app` | `playhound app` | `AppDetails` |
| Search | `Client::search` | `playhound search` | `Vec<AppOverview>` |
| Collections | `Client::list` | `playhound list` | `Vec<AppOverview>` |
| Reviews | `Client::reviews` | `playhound reviews` | `Page<Review>` |
| Suggestions | `Client::suggestions` | `playhound suggest` | `Vec<String>` |

Application details include localized descriptions, exact price micros, rating
statistics, install ranges, developer contact information, media URLs, content
rating, release information, version information, and release notes when Google
supplies them.

Search supports free/paid filtering and follows a continuation token when the
storefront provides one. The current storefront normally returns at most about
30 search results and may provide no continuation token, so a requested limit
is an upper bound rather than a guaranteed result count.

Collections support top-free, top-paid, and top-grossing lists, all modeled
categories, family age filters, and forward-compatible custom collection and
category values.

## Installation

### Async library only

The default feature builds the CLI. Disable it when PlayHound is used only as a
library:

```toml
[dependencies]
playhound = { version = "0.1", default-features = false }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

### Blocking library

```toml
[dependencies]
playhound = { version = "0.1", default-features = false, features = ["blocking"] }
```

### Library and CLI

```toml
[dependencies]
playhound = "0.1"
```

During local development, a path dependency can be used:

```toml
[dependencies]
playhound = { path = "../google-play-scraper", default-features = false }
```

## Feature flags

| Feature | Default | Purpose |
| --- | --- | --- |
| `cli` | Yes | Builds the `playhound` binary and its CLI/runtime dependencies. |
| `blocking` | No | Enables `playhound::blocking::Client` using Reqwest's native blocking client. |

The asynchronous `playhound::Client` is always available. Enabling
`blocking` adds the synchronous API; it does not replace the async client.

## Quick start

```rust
use playhound::{Client, SearchQuery};

#[tokio::main]
async fn main() -> playhound::Result<()> {
    let client = Client::new()?;
    let apps = client
        .search(SearchQuery::new("example query").limit(10))
        .await?;

    for app in apps {
        println!("{} ({})", app.title, app.app_id);
    }

    Ok(())
}
```

All network operations return `playhound::Result<T>`. Inputs are validated
before transmission.

## Async library API

### Application details

`Client::app` accepts an application ID directly:

```rust
use playhound::Client;

# async fn example() -> playhound::Result<()> {
let client = Client::new()?;
let app = client.app("com.example.app").await?;

println!("{}", app.overview.title);
println!("{:?}", app.version);
println!("{:?}", app.description);
# Ok(())
# }
```

Use `AppRequest` to override the locale for one request:

```rust
use playhound::{AppRequest, Client, Locale};

# async fn example() -> playhound::Result<()> {
let client = Client::new()?;
let request = AppRequest::new("com.example.app")
    .locale(Locale::new("de", "de")?);
let app = client.app(request).await?;
# Ok(())
# }
```

### Search

```rust
use playhound::{Client, PriceFilter, SearchQuery};

# async fn example() -> playhound::Result<()> {
let client = Client::new()?;
let apps = client
    .search(
        SearchQuery::new("example query")
            .limit(20)
            .price(PriceFilter::Free),
    )
    .await?;
# Ok(())
# }
```

`PriceFilter` values are:

- `PriceFilter::All`;
- `PriceFilter::Free`;
- `PriceFilter::Paid`.

The default search limit is 20. Limits must be nonzero and fit in a signed
32-bit wire value. Google may return fewer results than requested.

### Collections

```rust
use playhound::{AgeRange, Category, Client, Collection, ListQuery};

# async fn example() -> playhound::Result<()> {
let client = Client::new()?;
let apps = client
    .list(
        ListQuery::new(Collection::TopFree, Category::Education)
            .age(AgeRange::SixToEight)
            .limit(25),
    )
    .await?;
# Ok(())
# }
```

`ListQuery::default()` requests the top-free `APPLICATION` collection with
a limit of 50.

Supported collection values:

| Rust value | Wire/CLI value |
| --- | --- |
| `Collection::TopFree` | `topselling_free` |
| `Collection::TopPaid` | `topselling_paid` |
| `Collection::TopGrossing` | `topgrossing` |

Supported age values:

| Rust value | CLI values |
| --- | --- |
| `AgeRange::FiveAndUnder` | `five-and-under`, `5` |
| `AgeRange::SixToEight` | `six-to-eight`, `6-8` |
| `AgeRange::NineAndUp` | `nine-and-up`, `9+` |

The modeled category wire values are:

```text
GAME
FAMILY
APPLICATION
ANDROID_WEAR
ART_AND_DESIGN
AUTO_AND_VEHICLES
BEAUTY
BOOKS_AND_REFERENCE
BUSINESS
COMICS
COMMUNICATION
DATING
EDUCATION
ENTERTAINMENT
EVENTS
FINANCE
FOOD_AND_DRINK
HEALTH_AND_FITNESS
HOUSE_AND_HOME
LIBRARIES_AND_DEMO
LIFESTYLE
MAPS_AND_NAVIGATION
MEDICAL
MUSIC_AND_AUDIO
NEWS_AND_MAGAZINES
PARENTING
PERSONALIZATION
PHOTOGRAPHY
PRODUCTIVITY
SHOPPING
SOCIAL
SPORTS
TOOLS
TRAVEL_AND_LOCAL
VIDEO_PLAYERS
WATCH_FACE
WEATHER
GAME_ACTION
GAME_ADVENTURE
GAME_ARCADE
GAME_BOARD
GAME_CARD
GAME_CASINO
GAME_CASUAL
GAME_EDUCATIONAL
GAME_MUSIC
GAME_PUZZLE
GAME_RACING
GAME_ROLE_PLAYING
GAME_SIMULATION
GAME_SPORTS
GAME_STRATEGY
GAME_TRIVIA
GAME_WORD
```

`Collection` and `Category` are forward-compatible. Unknown printable
values can be parsed or created explicitly:

```rust
use playhound::{Category, Collection};

let collection: Collection = "future_collection".parse()?;
let category = Category::custom("FUTURE_CATEGORY")?;
# Ok::<(), playhound::Error>(())
```

### Reviews

```rust
use playhound::{Client, ReviewQuery, ReviewSort};

# async fn example() -> playhound::Result<()> {
let client = Client::new()?;
let page = client
    .reviews(
        ReviewQuery::new("com.example.app")
            .sort(ReviewSort::Newest)
            .page_size(50),
    )
    .await?;

for review in &page.items {
    println!("[{}/5] {}", review.score, review.user_name);
}
# Ok(())
# }
```

`ReviewSort` values are `Newest`, `Rating`, and `Helpfulness`. The
default is newest-first with a page size of 100.

Review pagination tokens are opaque. Pass them back unchanged:

```rust
use playhound::{Client, ReviewQuery};

# async fn example() -> playhound::Result<()> {
let client = Client::new()?;
let first = client
    .reviews(ReviewQuery::new("com.example.app").page_size(50))
    .await?;

if let Some(token) = first.next_page_token {
    let second = client
        .reviews(
            ReviewQuery::new("com.example.app")
                .page_size(50)
                .page_token(token),
        )
        .await?;
    println!("second page: {} reviews", second.items.len());
}
# Ok(())
# }
```

Do not log or interpret `PageToken` values. Its `Debug` implementation
reveals only the token length; `expose()` is available when the raw value is
needed for persistence or a subsequent request.

### Search suggestions

```rust
use playhound::{Client, SuggestionQuery};

# async fn example() -> playhound::Result<()> {
let client = Client::new()?;
let suggestions = client
    .suggestions(SuggestionQuery::new("example"))
    .await?;
# Ok(())
# }
```

A string can be passed directly when no per-request locale is needed:

```rust
# async fn example() -> playhound::Result<()> {
let client = playhound::Client::new()?;
let suggestions = client.suggestions("example").await?;
# Ok(())
# }
```

## Blocking library API

Enable the `blocking` feature and use `playhound::blocking::Client`. The
operation and request types are shared with the async client:

```rust
use playhound::{
    PriceFilter, ReviewQuery, SearchQuery,
    blocking::Client,
};

fn main() -> playhound::Result<()> {
    let client = Client::new()?;

    let apps = client.search(
        SearchQuery::new("example query")
            .limit(10)
            .price(PriceFilter::All),
    )?;

    let reviews = client.reviews(
        ReviewQuery::new("com.example.app").page_size(25),
    )?;

    println!("{} apps, {} reviews", apps.len(), reviews.items.len());
    Ok(())
}
```

The blocking implementation uses Reqwest's native blocking client and does not
create an async runtime internally. Do not call it directly from an async task;
use the async client or move blocking work to an appropriate blocking thread.

## Client configuration

Both clients provide the same builder options:

```rust
use std::{num::NonZeroU32, time::Duration};
use playhound::{Client, Locale, Proxy, RetryPolicy};

let client = Client::builder()
    .default_locale(Locale::new("en", "us")?)
    .proxy(Proxy::all("socks5h://127.0.0.1:1080")?)
    .use_system_proxy(false)
    .request_timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))
    .max_response_bytes(32 * 1024 * 1024)
    .requests_per_second(NonZeroU32::new(2).expect("nonzero"))
    .retry_policy(
        RetryPolicy::exponential(2)
            .base_delay(Duration::from_millis(250))
            .max_delay(Duration::from_secs(3))
            .honor_retry_after(true),
    )
    .build()?;
# Ok::<(), playhound::Error>(())
```

Default configuration:

| Setting | Default |
| --- | --- |
| Locale | `en` / `us` |
| Environment/system proxy discovery | Enabled |
| Request timeout | 30 seconds |
| Connection timeout | 10 seconds |
| Maximum response body | 32 MiB |
| Client-side rate limit | Disabled |
| Retries | Disabled |
| TLS certificate verification | Enabled |

Timeouts and the maximum response size must be nonzero. A retry policy's maximum
delay must not be shorter than its base delay.

`danger_accept_invalid_certs(true)` disables server-certificate validation.
It is intentionally named as a dangerous operation and should be limited to
controlled troubleshooting. It should not be used in production.

## Localization

`Locale::new(language, country)` validates and normalizes its values:

```rust
use playhound::Locale;

let locale = Locale::new("ET", "EE")?;
assert_eq!(locale.language.as_str(), "et");
assert_eq!(locale.country.as_str(), "ee");
# Ok::<(), playhound::Error>(())
```

Language values use a nonempty BCP-47-style syntax. Country values must contain
exactly two ASCII letters. Locale selection controls the requested storefront
language and country; it does not guarantee that every returned field is
translated.

Set a default locale on the client and override it on individual request types
with their `.locale(...)` method.

## Proxies

Supported proxy URL schemes:

- `http://`;
- `https://`;
- `socks5://`;
- `socks5h://` for proxy-side DNS resolution.

Use `Proxy::all`, `Proxy::http`, or `Proxy::https`:

```rust
use playhound::{Client, Proxy};

let client = Client::builder()
    .proxy(Proxy::http("http://127.0.0.1:8080")?)
    .proxy(Proxy::https("socks5h://127.0.0.1:1080")?)
    .build()?;
# Ok::<(), playhound::Error>(())
```

With no explicit proxy, Reqwest discovers the standard `HTTP_PROXY`,
`HTTPS_PROXY`, `ALL_PROXY`, and `NO_PROXY` variables, including lowercase
variants.

Adding any explicit proxy switches the client to the explicit proxy set and
disables automatic environment proxy discovery. Scheme-specific rules take
precedence over an all-traffic rule. Adding another proxy for the same scope
replaces the previous rule.

Call `.use_system_proxy(false)` to guarantee that environment proxies are not
used:

```rust
let client = playhound::Client::builder()
    .use_system_proxy(false)
    .build()?;
# Ok::<(), playhound::Error>(())
```

Proxy credentials may be embedded in the URL:

```text
http://username:password@proxy.example:8080
socks5h://username:password@proxy.example:1080
```

PlayHound redacts credentials from proxy `Debug` output and credential-bearing
transport errors. Avoid putting credentials in source code, shell history, or
committed configuration.

## Retries and rate limiting

Retries are disabled by default. Enable a bounded exponential policy explicitly:

```rust
use playhound::{Client, RetryPolicy};

let client = Client::builder()
    .retry_policy(RetryPolicy::exponential(3))
    .build()?;
# Ok::<(), playhound::Error>(())
```

Only transient read-only failures are retried: transport failures, rate limits,
and service-unavailable responses. Other HTTP statuses, invalid inputs, parsing
errors, and upstream response drift are returned immediately.

A valid server `Retry-After` delay is honored by default and capped by the
configured maximum delay. Set `.honor_retry_after(false)` to always use local
backoff.

Client-side throttling uses a burst capacity of one:

```rust
use std::num::NonZeroU32;

let client = playhound::Client::builder()
    .requests_per_second(NonZeroU32::new(2).expect("nonzero"))
    .build()?;
# Ok::<(), playhound::Error>(())
```

Rate limiting is per client instance. Clones of a client share its configuration
and limiter.

## Result models

All public result models implement `Serialize` and `Deserialize`. They are
marked `#[non_exhaustive]` so fields and variants can be added compatibly.

### `AppOverview`

Search and collection results contain:

- validated `app_id`;
- localized `title`;
- canonical `store_url`;
- optional icon, developer, and developer ID;
- optional numeric and localized score;
- optional exact `Money`;
- optional free/paid determination;
- optional localized summary.

### `AppDetails`

`AppDetails` contains an `overview: AppOverview` plus optional detailed
metadata:

- plain and HTML descriptions;
- install text and min/max install counts;
- rating/review counts and star histogram;
- availability and in-app-purchase declaration;
- required Android version;
- developer email, website, address, and privacy policy;
- genre and genre ID;
- header image, screenshots, and video;
- content rating;
- release date and update timestamp;
- version and recent changes;
- additional comments.

Optional fields are `None` when Google omits them. An empty vector represents
an available collection with no elements.

### `Money`

Prices use integer micros to avoid floating-point loss:

```rust
let price = playhound::Money::new(
    1_990_000,
    Some("USD".into()),
    Some("$1.99".into()),
);

assert_eq!(price.micros, 1_990_000);
assert!(!price.is_free());
assert_eq!(price.as_major_units(), 1.99);
```

`as_major_units()` is intended for display. Use `micros` for exact
comparison and storage.

### Reviews and pages

`Review` includes the review ID, user display information, timestamp, score,
title/body, optional `DeveloperReply`, application version, and helpful-vote
count.

`Page<T>` contains:

- `items: Vec<T>`;
- `next_page_token: Option<PageToken>`.

## Errors

Match individual `Error` variants when details are needed, or use
`Error::kind()` for stable broad handling:

```rust
use playhound::{Client, Error, ErrorKind};

# async fn example() -> playhound::Result<()> {
let client = Client::new()?;
match client.app("com.example.app").await {
    Ok(app) => println!("{}", app.overview.title),
    Err(Error::AppNotFound { app_id }) => {
        eprintln!("not found: {app_id}");
    }
    Err(error) if error.kind() == ErrorKind::RateLimited => {
        eprintln!("Google Play requested a slower rate");
    }
    Err(error) => return Err(error),
}
# Ok(())
# }
```

| `ErrorKind` | Meaning |
| --- | --- |
| `InvalidInput` | A request argument failed validation. |
| `NotFound` | The requested application was confirmed missing. |
| `RateLimited` | Google Play returned a rate-limit/service-unavailable response. |
| `HttpStatus` | Another non-success HTTP status was returned. |
| `Transport` | DNS, connection, TLS, timeout, or body-read failure. |
| `ResponseTooLarge` | The configured response-size limit was exceeded. |
| `UnexpectedResponse` | Google returned an unknown response shape. |
| `Parse` | Recognized response data could not be converted. |
| `Configuration` | Client construction or configuration failed. |

`UnexpectedResponse` is deliberately distinct from `NotFound`: upstream
format drift is not reported as a missing application.

## CLI installation

### Run from this repository

```console
cargo run -- --help
cargo run -- search --json "example query"
```

The `--` separates Cargo options from PlayHound options. Cargo may accept some
arguments without it, but using the separator is unambiguous and recommended.

### Build a standalone binary

```console
cargo build --release
./target/release/playhound --help
```

On Windows:

```console
target\release\playhound.exe --help
```

### Install from a local checkout

```console
cargo install --path .
playhound --help
```

### Install from crates.io

```console
cargo install playhound
```

## CLI command reference

Global options can appear before or after a subcommand.

### Application details

```console
playhound app <APP_ID>
playhound app --lang de --country de <APP_ID>
playhound app --json <APP_ID>
```

Human output is a compact summary. `--json` emits the complete serialized
`AppDetails` object.

### Search

```console
playhound search [OPTIONS] <QUERY>
```

Search-specific options:

| Option | Values | Default |
| --- | --- | --- |
| `--limit <N>` | Positive integer | 20 |
| `--price <PRICE>` | `all`, `free`, `paid` | `all` |

Examples:

```console
playhound search "example query"
playhound search --limit 10 --price free "example query"
playhound search --json "example query"
```

JSON output is an array of `AppOverview` objects.

### Collections

```console
playhound list [OPTIONS]
```

List-specific options:

| Option | Default |
| --- | --- |
| `--collection <VALUE>` | `topselling_free` |
| `--category <VALUE>` | `APPLICATION` |
| `--age <VALUE>` | No age filter |
| `--limit <N>` | 50 |

Examples:

```console
playhound list
playhound list --collection topselling_paid --category GAME --limit 25
playhound list --category FAMILY --age six-to-eight --json
```

Collection and category arguments also accept forward-compatible custom values.

### Reviews

```console
playhound reviews [OPTIONS] <APP_ID>
```

Review-specific options:

| Option | Values | Default |
| --- | --- | --- |
| `--sort <SORT>` | `newest`, `rating`, `helpfulness` | `newest` |
| `--page-size <N>` | Positive integer | 100 |
| `--page-token <TOKEN>` | Opaque token | None |
| `--pages <N>` | Positive integer | 1 |

Examples:

```console
playhound reviews <APP_ID>
playhound reviews --sort helpfulness --page-size 25 <APP_ID>
playhound reviews --page-size 50 --pages 3 --json <APP_ID>
playhound reviews --page-token "<OPAQUE_TOKEN>" --json <APP_ID>
```

The CLI follows at most `--pages` review pages and stops early when no token is
returned or a token repeats. JSON output is one accumulated array of reviews,
not a `Page<Review>` wrapper.

### Suggestions

```console
playhound suggest "example"
playhound suggest --json "example"
```

JSON output is an array of strings.

### Check all operations

```console
playhound check-all \
  --app-id <APP_ID> \
  --query "example query" \
  --limit 3
```

`check-all` runs app details, search, the default collection, one review page,
and suggestions. It prints one status per operation, or a structured report
with `--json`. It exits nonzero if any operation fails.

This command is intended for an explicitly manual compatibility check. It
contacts the live storefront and is never run by the test suite or CI.

## CLI configuration and environment variables

Global CLI options:

| Option | Environment variable | Default | Meaning |
| --- | --- | --- | --- |
| `--lang <CODE>` | `PLAYHOUND_LANG` | `en` | Response language. |
| `--country <CODE>` | `PLAYHOUND_COUNTRY` | `us` | Store country. |
| `--proxy <URL>` | `PLAYHOUND_PROXY` | None | Proxy all traffic. |
| `--http-proxy <URL>` | `PLAYHOUND_HTTP_PROXY` | None | Proxy HTTP traffic. |
| `--https-proxy <URL>` | `PLAYHOUND_HTTPS_PROXY` | None | Proxy HTTPS traffic. |
| `--no-system-proxy` | `PLAYHOUND_NO_SYSTEM_PROXY` | False | Disable environment proxy discovery. |
| `--requests-per-second <N>` | `PLAYHOUND_REQUESTS_PER_SECOND` | Unlimited | Client-side request rate. |
| `--timeout <SECONDS>` | `PLAYHOUND_TIMEOUT` | 30 | Complete request timeout. |
| `--connect-timeout <SECONDS>` | `PLAYHOUND_CONNECT_TIMEOUT` | 10 | Connection timeout. |
| `--max-response-bytes <N>` | `PLAYHOUND_MAX_RESPONSE_BYTES` | 33554432 | Response body limit. |
| `--retries <N>` | `PLAYHOUND_RETRIES` | 0 | Maximum transient retries. |
| `--insecure` | `PLAYHOUND_INSECURE` | False | Disable TLS certificate validation. |
| `--json` | None | False | Emit machine-readable output. |
| `-v`, `-vv` | `RUST_LOG` also supported | Warnings | Increase diagnostic verbosity. |

Standard `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, and `NO_PROXY`
variables are used only when explicit PlayHound proxy options are absent and
system proxy discovery is enabled.

Example:

```console
PLAYHOUND_COUNTRY=ee \
PLAYHOUND_LANG=et \
PLAYHOUND_PROXY=socks5h://127.0.0.1:1080 \
playhound search --json "example query"
```

Diagnostics are written to stderr. `RUST_LOG` can select a tracing filter:

```console
RUST_LOG=playhound=debug playhound search "example query"
```

### JSON output and errors

Successful `--json` output is written to stdout. Errors are written to stderr
as a single JSON object:

```json
{
  "error": "application not found: com.example.missing",
  "kind": "NotFound",
  "ok": false
}
```

CLI exit statuses:

| Status | Meaning |
| --- | --- |
| 0 | Success |
| 2 | Invalid input or client configuration |
| 3 | Application not found |
| 4 | Rate limited |
| 5 | HTTP, transport, TLS, timeout, or response-size failure |
| 6 | Upstream response drift or parsing failure |
| 1 | Other failure |

## Operational notes

### Upstream stability

Google Play's storefront response formats are not a supported API. A PlayHound
release can stop parsing an operation when the storefront changes. Treat
`ErrorKind::UnexpectedResponse` as a compatibility problem, not proof that an
application is missing.

Pin a PlayHound version in production automation, monitor failures, and update
deliberately.

### Result counts

All limits are maximum requested counts. Google can return fewer items because
of storefront caps, locale availability, filtering, exhausted pagination, or
response-shape changes.

### Responsible request behavior

- Cache data when practical.
- Avoid unnecessary concurrency.
- Configure `requests_per_second` for batch jobs.
- Keep retries bounded.
- Respect rate-limit responses.
- Do not retry parsing or invalid-input failures blindly.

### Security

- TLS validation is enabled by default.
- Response bodies are bounded to 32 MiB by default.
- Redirects are limited and restricted to the Google Play host.
- Proxy credentials are redacted from diagnostic formatting.
- No JavaScript from storefront pages is executed.

## Testing

Tests never contact Google Play or any other network service.

- Parser tests use synthetic wire fixtures.
- Normalized live snapshots are sanitized before being committed.
- Client tests inject in-memory async and blocking transports.
- Property tests feed malformed input to parsers.
- Repository policy tests reject inline production tests and known real fixture
  identities.
- Every test body lives in a separate file under `tests/`.

Run the full suite:

```console
cargo test --workspace --all-features --locked
```

Quality checks:

```console
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --no-default-features --lib --locked
cargo check --no-default-features --locked
cargo check --no-default-features --features blocking --locked
RUSTDOCFLAGS="-Dwarnings" cargo doc --all-features --no-deps --locked
cargo deny check
cargo llvm-cov --workspace --all-features --locked --summary-only
cargo package --all-features --locked
```

Maintainers can sanitize a normalized live capture with:

```console
cargo run -p playhound-xtask -- sanitize <kind> <input.json> <output.json>
```

`<kind>` is one of `app`, `search`, `list`, `reviews`, or
`suggestions`. Raw captures must not be committed.

## MSRV and platforms

- Rust edition: 2024.
- Minimum supported Rust version: 1.85.
- CI platforms: Linux, macOS, and Windows.
- TLS backend: Rustls through Reqwest.

The declared MSRV is checked in CI with all features enabled.

## License

PlayHound is licensed under the European Union Public Licence 1.2
(`EUPL-1.2`). See the `LICENSE` file distributed with the project and crate.
