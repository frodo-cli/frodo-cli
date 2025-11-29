# Dependency Bill of Materials (initial)

This SBOM tracks the crates currently declared in the workspace and the rationale for each. Versions were verified with `cargo search`/`cargo info` against crates.io before inclusion (Rust toolchain `1.87.0`).

| Crate | Version | Purpose | Notes |
| --- | --- | --- | --- |
| anyhow | 1.0.93 | Ergonomic error type for non-user-facing errors | Maintained, stable |
| clap | 4.5.53 | CLI argument parsing/validation | Using `derive` feature |
| color-eyre | 0.6.3 | Pretty error reports for users | Stable |
| serde | 1.0.228 | Serialization for config/data models | `derive` feature enabled |
| tokio | 1.48.0 | Async runtime | `macros`, `rt-multi-thread` features |
| tracing | 0.1.41 | Structured logging | |
| tracing-subscriber | 0.3.18 | Logging subscriber with env filter | `env-filter`, `fmt` |
| ratatui | 0.29.0 | Terminal UI rendering | Latest stable (0.30 is beta) |
| crossterm | 0.29.0 | Cross-platform terminal I/O backend | Matches ratatui stack |

Planned additions (will be added alongside tests when implemented): `reqwest`, `oauth2`, `sqlx` (SQLite with SQLCipher or app-layer AES-GCM), `keyring`, `async-openai`, `serde_json`, `insta`, `assert_cmd`, `httpmock`, `proptest`.

## Regeneration
- Produce a fresh SBOM tree (after adding crates): `cargo tree --workspace > target/sbom.txt` (commit the text file only when meaningful; keep this doc as the human-friendly summary).
- Verify versions before adding/upgrading: `cargo info <crate>` and read release notes/docs.rs for breaking changes.
- Keep `Cargo.lock` committed for reproducible builds.
