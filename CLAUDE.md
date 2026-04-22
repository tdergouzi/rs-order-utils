# CLAUDE.md

Guidance for Claude Code in this repository.

## Project

`rs-order-utils` — Rust implementation of Polymarket CLOB order utilities with EIP-712 typed-data signing. Exposes V1 (12-field CTF Exchange order at the top level) and V2 (11-field struct under `v2/` with `metadata` / `builder` / `timestamp` fields and `Poly1271` signatures).

Consumed by `rs-clob-client` (V1) and `rs-clob-client-v2` (V2) in sibling directories.

## Workflow

1. Analyze scope and risk.
2. Propose files to touch + key decisions; pause for confirmation on non-trivial changes.
3. Execute; surface issues that fall outside the agreed scope.

Skip propose/confirm for typos, formatting, or explicit unambiguous instructions.

## Build

`cargo check` · `cargo test` · `cargo fmt --all` · `cargo clippy`

Tests are self-contained (no `.env` / network required).

## Commits

Format: `<type>(<scope>): <subject>`

**Types**: `feat` · `fix` · `docs` · `style` · `refactor` · `test` · `chore` · `perf`

**Rules**:
- **Subject line ≤ 72 characters.** No descriptive body paragraphs.
- Lowercase, imperative mood (`add`, not `added`).
- Scope optional (e.g. `v2`, `builder`, `models`, `deps`).
- `Co-Authored-By:` trailer is allowed (and appropriate when pair-authored).

Examples:
- `feat(v2): add v2 module with 11-field ctf exchange order`
- `fix(builder): compute eip712 hash from 11-field struct`
- `test(v2): assert metadata and builder affect signed hash`
- `chore: bump version to 0.3.0-alpha.1`

## Simplicity

- Inline single-use logic; do not create helpers for one-time use.
- Extract a helper only when reused 3+ times or it encapsulates real complexity.
- Do not add fallbacks, validation, or error handling for scenarios that cannot happen.
