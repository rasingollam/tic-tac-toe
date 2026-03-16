use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use std::time::{Duration, Instant};
use tower_http::cors::{Any, CorsLayer};

pub mod engine;
pub mod models;
pub mod state;

use models::{BoardResponse, Move, ResetPayload, SessionQuery, SetDifficultyPayload};
use state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let app = Router::new()
        .route("/board", get(get_board))
        .route("/move", post(make_move))
        .route("/reset", post(reset_game))
        .route("/difficulty", post(set_difficulty))
        .layer(cors)
        .with_state(state.clone());

    // Session cleanup background task
    let cleaner_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 10)); // Every 10 mins
        loop {
            interval.tick().await;
            let mut games = cleaner_state.games.lock().await;
            let now = Instant::now();
            games.retain(|_, game| now.duration_since(game.last_active) < Duration::from_secs(3600)); // 1 hour TTL
        }
    });

    println!("Tic-Tac-Toe API running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn build_response(game: &models::GameState) -> Json<BoardResponse> {
    let check = engine::check_winner_detailed(&game.board);
    let (winner, win_line) = match check {
        Some((w, line)) => (Some(w), Some(line)),
        None => (None, None),
    };

    Json(BoardResponse {
        board: game.board,
        turn: game.turn,
        winner,
        win_line,
        score_player: game.score_player,
        score_bot: game.score_bot,
        score_draws: game.score_draws,
        difficulty: game.difficulty.clone(),
    })
}

async fn get_board(
    State(state): State<AppState>,
    Query(query): Query<SessionQuery>,
) -> Json<BoardResponse> {
    let mut games = state.games.lock().await;
    let game = games.entry(query.session_id.clone()).or_insert_with(models::GameState::new);
    game.last_active = Instant::now();
    build_response(game)
}

async fn set_difficulty(
    State(state): State<AppState>,
    Json(payload): Json<SetDifficultyPayload>,
) -> Json<BoardResponse> {
    let mut games = state.games.lock().await;
    let game = games.entry(payload.session_id.clone()).or_insert_with(models::GameState::new);
    game.last_active = Instant::now();
    game.difficulty = payload.difficulty;
    build_response(game)
}

async fn make_move(
    State(state): State<AppState>,
    Json(mv): Json<Move>,
) -> Json<BoardResponse> {
    // Artificial bot delay for better UX
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut games = state.games.lock().await;
    let game = games.entry(mv.session_id.clone()).or_insert_with(models::GameState::new);
    game.last_active = Instant::now();

    if mv.row > 2 || mv.col > 2 || game.board[mv.row][mv.col] != ' ' || engine::check_winner_simple(&game.board).is_some() || game.turn != 'X' {
        return build_response(game);
    }

    // Player 1 (X) move
    game.board[mv.row][mv.col] = 'X';
    
    if let Some('X') = engine::check_winner_simple(&game.board) {
        game.score_player += 1;
        return build_response(game);
    } else if engine::is_board_full(&game.board) {
        game.score_draws += 1;
        return build_response(game);
    }

    game.turn = 'O';

    // Bot move (O) delegate to engine
    engine::make_bot_move(game);
    
    game.turn = 'X';

    if let Some('O') = engine::check_winner_simple(&game.board) {
        game.score_bot += 1;
    } else if engine::is_board_full(&game.board) {
        game.score_draws += 1;
    }

    build_response(game)
}

async fn reset_game(
    State(state): State<AppState>,
    Json(payload): Json<ResetPayload>,
) -> Json<BoardResponse> {
    let mut games = state.games.lock().await;
    let game = games.entry(payload.session_id.clone()).or_insert_with(models::GameState::new);
    game.last_active = Instant::now();
    game.board = [[' '; 3]; 3];
    game.turn = 'X';
    
    build_response(game)
}
