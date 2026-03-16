# Sleek Tic-Tac-Toe

A modern Tic-Tac-Toe app implemented in Rust (backend) and Yew (frontend).

Project layout (workspace):

- `backend/` — Rust Axum API server (port 3000)
- `frontend/` — Yew WASM frontend (served with Trunk)

Quick start (requires `rustup`, `cargo`, and `trunk`):

1) Build and run the backend (from repo root):

```bash
cd backend
cargo run
```

2) Serve the frontend (in a separate terminal):

```bash
# Sleek Tic-Tac-Toe

A modern Tic‑Tac‑Toe application with a Rust backend (Axum) and a Yew WebAssembly frontend.

This repository is organized as a Cargo workspace so backend and frontend crates share a single dependency lockfile and build context.

**Project layout**

- `backend/` — Rust Axum API server. Provides HTTP endpoints on port `3000` by default.
- `frontend/` — Yew-based WebAssembly frontend, built and served with `trunk` (dev server on `127.0.0.1:8080`).

**Goals**

- Small, easy-to-run demo of a client-server Tic‑Tac‑Toe game
- Minimal dependencies and clear workspace structure
- Reproducible builds via a single `Cargo.lock` at the repository root

## Prerequisites

- Rust toolchain (stable) with `rustup` and `cargo`
- `trunk` to build/serve the Yew frontend (install with `cargo install trunk`)
- `wasm32-unknown-unknown` target for Rust (install with `rustup target add wasm32-unknown-unknown`)

## Quick setup

1. Clone the repository (if not already):

```bash
git clone <repo-url>
cd tic-tac-toe
```

2. Install prerequisites (example):

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk # only if you plan to run the frontend dev server
```

## Development — run locally

Open two terminals and run backend + frontend concurrently.

- Terminal A — backend

```bash
cd backend
cargo run
```

This starts the Axum server on `http://0.0.0.0:3000`.

- Terminal B — frontend (dev server with live reload)

```bash
cd frontend
trunk serve --open
```

Trunk will build the WASM bundle and open the app at `http://127.0.0.1:8080` (or a nearby port if occupied). The frontend expects the API at `http://localhost:3000`.

## Production / build

- Build the backend release binary

```bash
cd backend
cargo build --release
```

- Build the frontend for production (generates `dist/`)

```bash
cd frontend
trunk build --release
```

You can then serve the contents of `frontend/dist/` from any static file host (NGINX, S3, simple `python3 -m http.server` for testing):

```bash
cd frontend/dist
python3 -m http.server 8080
```

## Notes on Cargo workspace and lockfiles

- The repository root contains `Cargo.toml` configured as a workspace and a single `Cargo.lock`. This is the recommended practice for multi-crate Rust repositories: one workspace manifest plus one lockfile ensures consistent dependency resolution across members.
- Do not check in per-crate `Cargo.lock` files for workspace members — the workspace `Cargo.lock` is authoritative (this repo ignores `frontend/Cargo.lock`).

## Code quality & tooling

- Format Rust code: `cargo fmt --all`
- Run clippy checks: `cargo clippy -p backend -- -D warnings` (install `clippy` via `rustup component add clippy`)

## Useful commands

- Build everything (workspace root):

```bash
cargo build --workspace
```

- Run backend tests (if any):

```bash
cd backend
cargo test
```

## Troubleshooting

- If `trunk` fails with errors from native networking crates when running from the repository root, ensure you run `trunk` inside the `frontend/` directory — Trunk will only build the frontend crate.
- If you see CORS errors in the browser, verify the backend is running and that CORS is allowed (the server uses a permissive CORS policy for local development).

## Contributing

- Open an issue or submit a PR. Keep changes small and focused. Run `cargo fmt` and include tests where helpful.

## License

- This project is provided as-is for demo / educational purposes. Add a license file if you plan to distribute it.

If you want, I can also add a small `Makefile` or `dev` script to start both services concurrently. Would you like that?

