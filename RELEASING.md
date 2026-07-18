# Releasing

Pushing a version tag matching `v*` automatically validates and publishes the
corresponding crate through `.github/workflows/publish.yml`.

Publishing is permanent: a crates.io version cannot be overwritten or deleted.
Do not push a release tag until the release commit has passed review and the
manual live compatibility check.

## One-time bootstrap

Trusted Publishing can be enabled only after the first crate version exists.

1. Log in to crates.io, verify the maintainer email, and confirm ownership of
   the `playhound` crate name.
2. Complete the local release gate below.
3. Publish version `0.1.0` manually:

   ```text
   cargo publish --all-features --locked
   ```

4. In the crates.io settings for `playhound`, add a GitHub Actions trusted
   publisher with:

   - repository owner: `RankoR`;
   - repository: `playhound`;
   - workflow: `publish.yml`;
   - environment: leave empty.

5. In GitHub, create a repository ruleset for `v*` tags. Restrict tag creation,
   updates, and deletion to trusted maintainers.
6. Push `v0.1.0`. The workflow recognizes that the version already exists and
   succeeds only if the verified crates.io package contents match the package
   built from the tag. Differences in gzip encoding between Cargo versions do
   not cause a false failure.

No long-lived crates.io token is stored in GitHub. The official crates.io
authentication action exchanges GitHub's OIDC identity for a short-lived token
and revokes it after the job.

## Release checklist

1. Update the version in `Cargo.toml`.
2. Move release notes from `[Unreleased]` to a dated version in
   `CHANGELOG.md`.
3. Confirm the MSRV and all public API changes.
4. Run the local gate:

   ```text
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
   cargo test --workspace --all-features --locked
   cargo test --no-default-features --lib --locked
   cargo check --no-default-features --locked
   cargo check --no-default-features --features blocking --locked
   cargo +1.88.0 check --all-features --locked
   RUSTDOCFLAGS="-Dwarnings" cargo doc --all-features --no-deps --locked
   cargo deny check
   cargo package --all-features --locked
   cargo package --all-features --locked --list
   ```

5. Run the explicitly manual live `check-all` smoke test documented in the
   README.
6. Commit and push the release commit to `main`; wait for CI to pass.
7. Create an annotated tag that exactly matches the manifest version:

   ```text
   git tag -s vX.Y.Z -m "playhound vX.Y.Z"
   git push origin vX.Y.Z
   ```

8. Monitor the `Publish to crates.io` workflow. It verifies that:

   - the tag is exactly `v<manifest-version>`;
   - the tagged commit is reachable from `main`;
   - formatting, linting, tests, docs, feature boundaries, and MSRV pass;
   - the crates.io package builds and verifies;
   - an already-published version has the same verified package contents.

To validate an existing release again without moving its immutable tag, run
the `Publish to crates.io` workflow manually from `main` and enter the existing
tag (for example, `v0.1.0`). Manual runs never publish a missing version.

9. After crates.io publication succeeds, create the corresponding GitHub
   Release from the existing tag and use the matching changelog section as its
   release notes.

Never move, reuse, or force-update a published version tag. If a release is
defective, fix it in a new version. Yank the defective version only when
preventing new dependency resolution is necessary.
