# Contributing

Thank you for helping improve PlayHound.

## Development

The minimum supported Rust version is 1.88. Use a current stable toolchain for
normal development.

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --no-default-features --lib
cargo doc --all-features --no-deps
```

Tests must be deterministic and must never contact Google Play or any other
network service. Use the fake transport and sanitized fixtures. Every test must
live in a separate file under `tests/`; production source files may only attach
those files with `#[path = ...] mod tests`.

Never commit real application IDs, application or developer names, user names,
emails, image URLs, review IDs, continuation tokens, or raw captured pages.
Fixture data uses `com.example.*`, `example.invalid`, and generic display text.

Keep parsing, wire construction, transport, models, clients, and CLI behavior in
separate modules. Public API changes need documentation, black-box tests, and a
changelog entry.

## Pull requests

Keep changes focused. Explain user-visible behavior, tests, and any expected
upstream response-shape assumptions. By contributing, you agree that your
contribution is licensed under EUPL-1.2.
