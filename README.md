# Sleek Tic-Tac-Toe

A modern, responsive, glassmorphism-styled Tic-Tac-Toe game built with a minimal technology stack.

## Tech Stack
- **Backend:** Rust + Axum (Async memory-based web framework)
- **Frontend:** HTML, Vanilla CSS & JavaScript (fetch API)
- **Architecture:** Client-Server model with Session support for multi-user, cross-browser concurrency.
- **AI bot:** Simple built-in Randomized Bot opponent utilizing `rand` to play back against you.

## Features
- Sleek gradients & distinct neon-accent player colors (`#38bdf8` and `#f472b6`)
- Multi-session concurrency over the simple Rust Mutex `std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, GameState>>>`.
- Live DOM validation restricting cursor options when hovering unplayable grids!

## How to play
1. Run the Rust Backend (Ensure `cargo/rustup` is configured correctly):
```bash
cargo run
```
Provides endpoints over `http://localhost:3000`.

2. Serve the frontend in a parallel terminal:
```bash
python3 -m http.server 8080
```

3. Visit your simple local network app on `http://localhost:8080`. Copy your session-assigned query url string to resume anywhere without conflicting states!
