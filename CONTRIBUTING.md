# Contributing

Thanks for taking the time to improve dir-vigil.

## Development setup

Requirements:

- Rust stable (edition 2021) and `cargo`
- Node.js >= 18.12 (Node 22 recommended) and Yarn 1.x
- Docker (optional, for container builds)

```bash
# backend
cd backend
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test

# frontend
cd frontend/h5
cp .env.example .env   # optional; defaults work without it
yarn install --frozen-lockfile
yarn typecheck
yarn build
```

## Pull requests

- Keep changes focused and update README.md / README_ZH-CN.md together when
  user-facing behaviour changes.
- Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  `yarn typecheck`, and `yarn build` before opening a PR.
- Use Conventional Commits-style messages (e.g. `feat:`, `fix:`, `docs:`,
  `chore:`), matching the existing history.
- Never commit `user_secret.enc`, `DIR_VIGIL_SECRET_KEY`, alert URLs, logs,
  `dist/`, `node_modules/`, or build binaries.

## Safety

This project deletes directories by design. Tests and manual verification must
use temporary directories only (`mktemp -d` or a disposable Docker volume).
Never point the test instance at real data.
