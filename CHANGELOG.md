# Changelog

All notable changes are documented here. This project follows Semantic Versioning.

## [Unreleased]

- Discover current storefront search results across ordered page sections so
  featured content preceding the result list does not break parsing.
- Add tag-triggered crates.io Trusted Publishing with OIDC authentication,
  release validation, and idempotent package checksum verification.

## [0.1.0] - 2026-07-18

- Initial async-first Rust library.
- Application details, search pagination and price filters, collections and age
  filters, review pagination and sorting, and search suggestions.
- Optional native blocking client.
- Explicit and environment proxy support, bounded responses, throttling, optional
  retries, strict TLS defaults, typed errors, structured tracing, and JSON output.
- Companion CLI with one command per operation and `check-all`.
- Compatibility with the current `/store/search` response and its server-side result
  cap, while retaining legacy continuation parsing for compatible responses.
