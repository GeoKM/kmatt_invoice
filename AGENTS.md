# Repository Guidelines

## Project Structure & Module Organization
- Core code in `src/`: `main.rs` boots the GUI; `gui.rs` houses egui screens and state; `database.rs` persists data to `database.json` and rotates `*.bak` backups; `pdf_generator.rs` formats invoices; `models.rs` defines DTOs; `utils.rs` holds helpers.
- Build artifacts land in `target/`; runtime data (`database.json` plus backups) live in the repo root by default.
- No test suite yet; add new tests under `src/` or `tests/` following Rust’s conventions.

## Build, Test, and Development Commands
- `cargo build` — debug build for day-to-day work.
- `cargo build --release` — optimized binary for operators (`target/release/kmatt_invoice`).
- `cargo run` — launches the GUI; prints "Launching GUI..." in the terminal.
- `cargo test` — runs any added unit/integration tests (currently none).
- `cargo fmt` / `cargo clippy --all-targets --all-features` — format and lint before submitting changes.

## Coding Style & Naming Conventions
- Rust 2021 edition; use `rustfmt` defaults (4-space indent, snake_case for functions/vars, UpperCamelCase for types).
- Keep egui UI code predictable: group related widgets in helper functions and avoid long, monolithic `ui` blocks.
- Prefer explicit error messages (see `DatabaseError`) and early returns for validation paths.
- Place new configuration or persisted data alongside `database.json` unless a config flag is introduced.

## Testing Guidelines
- Add unit tests near modules they cover (e.g., `src/database.rs` tests for backup rotation, invoice numbering, and JSON serialization).
- For GUI changes, include a short note in PRs on manual checks (e.g., "Created invoice, verified PDF saved and backup rotated").
- Aim to keep `cargo test` passing and free of `clippy` warnings; no coverage threshold enforced yet.

## Commit & Pull Request Guidelines
- Follow concise, action-first commit messages observed in history (e.g., "Add backup rotation", "Fix PDF totals alignment").
- One logical change per commit when possible; include context in the body if behavior changes (paths, env vars, migration notes).
- PRs should describe user-facing changes, steps to reproduce, and any manual testing performed; add screenshots/gifs for GUI updates and mention PDF output impacts.

## Security & Configuration Tips
- `database.json` and backups may contain customer data; do not commit real data. Add new ignore patterns if you generate additional artifacts.
- When distributing binaries, prefer `--release` builds and remind operators to keep backup files secure or relocate them to a protected directory.
