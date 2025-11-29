# frodo-cli ![CI](https://github.com/frodo-cli/frodo-cli/actions/workflows/ci.yml/badge.svg)

Local-first developer companion CLI. Current capabilities:
- Encrypted storage (AES-GCM, keys in OS keychain)
- Tasks: `task add/list/done` (encrypted)
- Ask: `ask "<prompt>"` (OpenAI if configured, otherwise Echo)
- TUI: `tui` (navigate j/k, mark done with `d`, quit with `q`/Esc)
- Sync: `sync` (GitHub/Jira pulls; push on `--apply`; otherwise dry-run)
- Health/config: `health`, `config init`
- Self-update: `self-update` (checks/downlods latest GitHub release; `--check` for dry-run)

## Quickstart
```bash
cargo run -- config init          # create ~/.config/frodo/config.toml if missing
cargo run -- task add "example"   # add a task
cargo run -- tui                  # view tasks, j/k to move, d to mark done
cargo run -- ask "what next?"     # uses tasks as context
cargo run -- sync                 # dry-run pull/push (GitHub/Jira/noop from config)
cargo run -- sync --apply         # applies push (creates issues)
cargo run -- self-update --check  # check for newer release
cargo run -- self-update          # download & replace binary
cargo run -- health               # check encrypted store/keyring
```

### Configure OpenAI (optional)
Set `OPENAI_API_KEY` or in `~/.config/frodo/config.toml`:
```toml
[openai]
api_key = "sk-..."
model = "gpt-4o-mini"
```

### Configure Jira / GitHub (for upcoming sync)
```toml
[jira]
site = "https://your-site.atlassian.net"
project_key = "PROJ"
api_token = "token"
email = "you@example.com"

[github]
owner = "your-org"
repo = "your-repo"
token = "ghp_..."
```
