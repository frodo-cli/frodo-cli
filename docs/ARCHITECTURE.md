# Architecture Overview

Frodo CLI is a local-first, encrypted developer companion that stays usable offline, auto-syncs when online, and speaks with a human-like agent. The codebase is organized as a Rust workspace to keep CLI/TUI, domain, storage, and integrations modular and testable.

## Layering
- **CLI/TUI** (`crates/frodo-cli/src`): command parser (Clap) plus Ratatui-based UI. Thin layer that delegates to services; defaults to `frodo tui`. Includes `frodo health` to verify encrypted storage/keyring availability, `config init` to scaffold `~/.config/frodo/config.toml` (platform aware), `frodo ask` (prefers OpenAI when configured, falls back to `EchoAgent`), and `frodo task {add,list,done}` backed by the encrypted store; TUI renders the local task list snapshot.
- **Core domain & contracts** (`crates/frodo-core`): shared models and traits (agent interface, secure store contract/stub, task model/repo trait); future home for task/conversation/workspace models and prioritization logic.
- **Storage** (`crates/frodo-storage`): encrypted local store (AES-GCM with keys in OS keychain; future SQLite + SQLCipher or app-layer AES-GCM) with a change journal for offline edits; key wrapping via OS keychain; migration tooling.
- **Sync** (planned `crates/sync`): reconciles local journal with Jira/GitHub Issues; conflict policy (latest-write with surfaced diffs); offline queue and retry.
- **Agents** (`crates/frodo-agent`): provider implementations (OpenAI chat completions, rustls transport); future local/offline providers and tool-calling.
- **Tasks** (`crates/frodo-task`): task repository implementation on top of the encrypted store.
- **Integrations** (planned `crates/integrations`): Jira and GitHub adapters using HTTP clients with mocked tests; auth via GitHub OAuth (device flow) and Jira tokens.
- **Plugins** (future): Wasm/exec hooks for new tools/providers with capability gating.

## Data & Security
- Local-first: all state (tasks, conversations, cached issues) lives locally and works offline.
- Encryption: data encrypted at rest; data key stored wrapped in OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service/KWallet).
- Sync: best-effort, resumable; no uploads without explicit scope (Jira projects, GitHub repos).
- Logs: structured tracing without persisting plaintext prompts or secrets; redaction pass before sending context to remote agents.

## Cross-Platform
- Target OS: Windows (MSVC), macOS (Intel/ARM), Linux (x86_64/ARM).
- Terminal stack: `crossterm` + `ratatui` for parity across shells and platforms.
- Paths/config: store config in `~/.frodo/config.toml` (platform-appropriate dirs via `dirs` crate later).

## Folder Structure (initial)
- `Cargo.toml` — workspace manifest and shared dependency versions.
- `crates/frodo-core` — shared contracts (agent, secure store stub, task model).
- `crates/frodo-storage` — concrete encrypted store implementations.
- `crates/frodo-agent` — agent providers (OpenAI now; local/offline later).
- `crates/frodo-task` — task repository implementations.
- `crates/frodo-cli` — binary crate for CLI/TUI entry.
- `docs/` — architecture and SBOM (dependency bill of materials).
- `AGENTS` — agent behavior and safety contract.

Future crates will land under `crates/` as the domain/storage/sync/agent/integration layers are implemented.

## Testing Strategy (high level)
- Unit tests per module (logic, parsing).
- Integration tests for CLI flows (using `assert_cmd`) and TUI snapshots (`insta`).
- Contract tests for provider/integration traits with mocked HTTP.
- Property tests for prioritization and merge/conflict handling.

## Version Discipline
- Use latest maintained releases; avoid betas unless required.
- Record checked versions in SBOM; re-check docs before upgrades.
- MSRV tracked in `Cargo.toml` workspace metadata.
