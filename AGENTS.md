# Frodo Agents

This file defines how Frodo’s AI agents work, what they can access, and how to keep them up to date, safe, and swappable. Treat this as the single source of truth for agent behavior and interfaces.

## Goals
- Feel like a human teammate in the CLI/TUI: concise, empathetic, and actionable.
- Local-first: fully usable offline with a local model; auto-sync context when online.
- Secure by default: all persisted data encrypted; keys in OS keychain; no silent uploads.
- Pluggable providers: OpenAI default, easily swap to other hosted or local models.
- Cross-platform: Windows/macOS/Linux parity; no platform-specific assumptions.

## Capabilities
- `ask`: answer questions using project context (tasks, git status/diff, chat transcripts).
- `chat`: threaded conversations with team members; agent can join and summarize.
- `task-help`: generate or refine task descriptions, acceptance criteria, priorities.
- `sync-aware`: may explain what changed after Jira/GitHub pulls/pushes.
- `review-lite`: shallow reasoning about risks or missing info; never auto-commits.

## Context Sources
- Workspace: repo metadata, git status/diff, recent commits (respect ignore rules).
- Tasks: local task DB (encrypted), Jira/GitHub Issues (cached/synced).
- Conversations: local transcripts (encrypted), recent team messages.
- Config: active workspace, user identity (from GitHub OAuth), feature flags.

## Data Flow (per request)
1) Gather context (bounded by size limits) via `ContextBuilder`.
2) Build prompt with persona + task + tools availability.
3) Stream request to provider (OpenAI by default) with tool-calling enabled.
4) Handle tool calls (local functions: task lookup, search, git summary, etc.).
5) Stream final answer to TUI; write transcript to encrypted store.
6) Redact secrets in logs/telemetry (telemetry is off by default).

## Core Abstractions
- `Agent`: async interface `ask(request, context) -> Stream<ResponseChunk>`.
- `AgentProvider`: factory for concrete providers; hides transport/auth.
- `Tool`: typed callable exposed to the model (e.g., `find_task`, `search_repo`).
- `ContextBuilder`: pluggable collectors; enforce byte/time budgets.
- `Memory`: conversation history store with TTL and size caps; persisted encrypted.
- `Policy`: safety hooks (max tokens, rate limits, allowlist of tools).

## Providers
- Default: OpenAI (use latest `async-openai` crate version; verify against docs before each update).
- Local: `llama.cpp`/GGML-backed provider (binary or library mode), for offline mode.
- Stub: deterministic fake for tests.
- Config: endpoint, model id, api key, timeouts, max tokens, temperature; pulled from config file + env; secrets kept in keychain when possible.
- Transport: `reqwest` with strict TLS; HTTP proxy honors env vars; retries with jitter.

## Prompting & Outputs
- Persona: “friendly senior teammate” with concise, stepwise reasoning when useful; avoid verbosity.
- Roles: system (guardrails), user (question), tools (function calls), assistant (final).
- Tool calling: prefer structured calls over free-form text; validate arguments before execution.
- Formatting: short bullet answers by default; code blocks when returning code; avoid leaking raw JSON unless asked.

## Offline & Sync Behavior
- If provider unavailable, fall back to local model; surface degraded-mode notice.
- Context sync (tasks/conversations) is best-effort; agent should not promise remote updates until confirmed.
- Conflict handling: prefer latest write but surface conflicts to user with minimal diff.

## Security & Privacy
- Encryption: all agent transcripts, cached context, and secrets encrypted at rest using AES-GCM; data keys wrapped by the OS keychain (Keyring) and never logged.
- No unapproved uploads: only the minimal context required for the request is sent to remote providers.
- Redaction: strip obvious secrets (tokens, keys, emails) from prompts/logs.
- Logging: structured debug logs, but never persist full prompts in plaintext.

## Testing
- Contract tests per provider using shared suites (prompts, tool-call shapes, error handling).
- Offline/local tests using stub provider; assert prompt assembly and tool-call selection.
- Snapshot tests for TUI render of streaming responses.
- Property tests for context budget enforcement and redaction.
- Integration tests around encryption round-trips and keychain interactions.

## Observability
- Tracing spans around context building, provider calls, tool execution, and streaming.
- Metrics: request counts, latencies, tool-call counts, fallback occurrences (no PII).

## Extensibility
- New providers: implement `AgentProvider`; register via config/plugins.
- New tools: add capability registry; tools declare input schema, cost, and safety level.
- Plugins: future support via Wasm/exec hooks to add tools while sandboxing execution.

## Implementation Notes (version discipline)
- Before adding/upgrading any agent-related dependency, check the latest docs/releases (crates.io + provider docs) and record the version + date in the PR/commit message.
- Prefer stable, maintained crates; avoid unmaintained forks. Re-evaluate periodically.
- Keep MSRV aligned with the most demanding dependency; document it in `Cargo.toml`.
