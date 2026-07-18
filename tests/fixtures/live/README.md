# Sanitized live snapshots

These normalized JSON snapshots were captured from the public Google Play
storefront on 2026-07-18 with the `en`/`us` locale, then processed by:

```console
cargo run -p playhound-xtask -- sanitize <kind> <input> <output>
```

Application, developer, and reviewer identities; package names; free-form
listing and review text; media; URLs; opaque tokens; and review timestamps and
versions are replaced with generic values. Raw captures are never committed.

These files verify the public model and serialization contract. Synthetic wire
fixtures in `tests/support/fixtures.rs` exercise the HTML and batchexecute
parsers without any network access.
