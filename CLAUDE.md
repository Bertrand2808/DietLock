## Commands

Toolchain pinned to Rust 1.99.0 in `rust-toolchain.toml` (rustup installs it automatically). Edition 2024.

```bash
cargo fmt --all --check # Format
cargo clippy --all-targets -- -D warnings # Lint (warnings are errors)
cargo test # All tests
cargo test -p dietlock-core # (or `-p dietlock-store`, `-p dietlock`) One crate
cargo test -p dietlock-core weight_bounds # (substring match on test name)  One test
cargo deny check licenses # (requires cargo-deny installed locally)  Licence check
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy, and test on macOS, Windows and Ubuntu, then cargo-deny on Linux. Run the first three locally before pushing.

There is no runnable UI yet: `src/index.html` is an empty stub and `src-tauri/src/lib.rs` registers no commands.

## Architecture

- **`dietlock-core`**: pure business logic. No Tauri, no SQLite, no clock. `Day` (strict `YYYY-MM-DD`), `WeighIn`, `Intake` are constructed through validating `new()` functions. "Today" must be passed in by the caller. Files: `error.rs` (`ValidationError`, `check_range`), `day.rs`, `weigh_in.rs`, `intake.rs`; `lib.rs` only declares modules and re-exports.
- **`dietlock-store`**: SQLite persistence via `rusqlite` (bundled). `Store` owns one connection, one row per date in `weigh_in` and `intake` tables. `save_*` takes a `replace` flag: when false and the date is filled, it returns `SaveError::AlreadyFilled(old)` so the caller can ask for confirmation (rule S3). Files: `lib.rs` (`Store`, thin delegating methods), `error.rs` (`StoreError`, `SaveError`), `schema.rs` (`SCHEMA_V1`, `migrate`), `weigh_in.rs` and `intake.rs` (SQL and row mapping as free functions on `&Connection`).
- **`src-tauri` (lib `dietlock_lib`)**: Tauri shell. `main.rs` only calls `dietlock_lib::run()`. This is where IPC commands will go once they exist; `frontendDist` points at `../src`.

### Invariants to keep in sync

- Value ranges are enforced twice: by `dietlock-core` constants (`WEIGHT_TENTH_KG_MIN/MAX`, `MACRO_G_MAX`, `WATER_ML_MAX`) and by `CHECK` clauses in `SCHEMA_V1` (`dietlock-store`). Changing a range means changing both, plus a test.
- Schema version lives in `PRAGMA user_version`. `schema::migrate` (called by `Store::init`) creates v1 from scratch when version is 0 and refuses any unknown version. A future migration needs a new branch in `migrate` and a bumped `SCHEMA_VERSION`.
- Weight is stored as integer tenths of kg; never store floats.

### Business rules

- Read `.claude/rules/metier.md` before changing any behaviour. It is the source of truth for the business rules.
- `CONTEXT.md` holds the ubiquitous language: use its terms in code and docs.

## Conventions

- All code, comments, error messages and config comments are in English. Do not add French to code. Domain terms come from `CONTEXT.md`.
- Tests are inline `#[cfg(test)] mod tests` in the module they cover. Shared helpers live in the `tests` module of each crate's `lib.rs` (`pub(crate)`); tests spanning both entities stay there. Store tests use `Store::open_in_memory()`.
- Dependency licences must be on the allowlist in `deny.toml` (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0, or a per-crate exception). A new dependency with another licence fails CI.
- Commit messages follow Conventional Commits with a scope, e.g. `feat(store): ...`.
