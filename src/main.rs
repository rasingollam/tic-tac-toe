use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex as AsyncMutex;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Clone)]
struct GameState {
    board: [[char; 3]; 3],
    turn: char,
    last_active: Instant,
    difficulty: Difficulty,
    score_player: u32,
    score_bot: u32,
    score_draws: u32,
}

impl GameState {
    fn new() -> Self {
        Self {
            board: [[' '; 3]; 3],
            turn: 'X',
            last_active: Instant::now(),
            difficulty: Difficulty::Hard,
            score_player: 0,
            score_bot: 0,
            score_draws: 0,
        }
    }
}

#[derive(Clone)]
struct AppState {
    games: Arc<AsyncMutex<HashMap<String, GameState>>>,
}

#[derive(Deserialize)]
struct SessionQuery {
    session_id: String,
}

#[derive(Deserialize)]
struct Move {
    session_id: String,
    row: usize,
    col: usize,
}

#[derive(Deserialize)]
struct SetDifficultyPayload {
    session_id: String,
    difficulty: Difficulty,
}

#[derive(Serialize)]
struct BoardResponse {
    board: [[char; 3]; 3],
    turn: char,
    winner: Option<char>,
    win_line: Option<Vec<(usize, usize)>>,
    score_player: u32,
    score_bot: u32,
    score_draws: u32,
    difficulty: Difficulty,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        games: Arc::new(AsyncMutex::new(HashMap::new())),
    };

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

fn build_response(game: &GameState) -> Json<BoardResponse> {
    let check = check_winner_detailed(&game.board);
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
    let game = games.entry(query.session_id.clone()).or_insert_with(GameState::new);
    game.last_active = Instant::now();
    build_response(game)
}

async fn set_difficulty(
    State(state): State<AppState>,
    Json(payload): Json<SetDifficultyPayload>,
) -> Json<BoardResponse> {
    let mut games = state.games.lock().await;
    let game = games.entry(payload.session_id.clone()).or_insert_with(GameState::new);
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
    let game = games.entry(mv.session_id.clone()).or_insert_with(GameState::new);
    game.last_active = Instant::now();

    if mv.row > 2 || mv.col > 2 || game.board[mv.row][mv.col] != ' ' || check_winner_simple(&game.board).is_some() || game.turn != 'X' {
        return build_response(game);
    }

    // Player 1 (X) move
    game.board[mv.row][mv.col] = 'X';
    
    if let Some('X') = check_winner_simple(&game.board) {
        game.score_player += 1;
        return build_response(game);
    } else if is_board_full(&game.board) {
        game.score_draws += 1;
        return build_response(game);
    }

    game.turn = 'O';

    // Bot move (O)
    let move_pos = match game.difficulty {
        Difficulty::Easy => get_random_move(&game.board),
        Difficulty::Medium => {
            let mut rng = rand::rng();
            if rng.random_bool(0.5) { 
                get_random_move(&game.board)
            } else {
                get_best_move(&game.board)
            }
        },
        Difficulty::Hard => get_best_move(&game.board),
    };

    if let Some((r, c)) = move_pos {
        game.board[r][c] = 'O';
    }
    
    game.turn = 'X';

    if let Some('O') = check_winner_simple(&game.board) {
        game.score_bot += 1;
    } else if is_board_full(&game.board) {
        game.score_draws += 1;
    }

    build_response(game)
}

#[derive(Deserialize)]
struct ResetPayload {
    session_id: String,
}

async fn reset_game(
    State(state): State<AppState>,
    Json(payload): Json<ResetPayload>,
) -> Json<BoardResponse> {
    let mut games = state.games.lock().await;
    let game = games.entry(payload.session_id.clone()).or_insert_with(GameState::new);
    game.last_active = Instant::now();
    game.board = [[' '; 3]; 3];
    game.turn = 'X';
    
    build_response(game)
}

fn check_winner_simple(board: &[[char; 3]; 3]) -> Option<char> {
    check_winner_detailed(board).map(|(w, _)| w)
}

fn check_winner_detailed(board: &[[char; 3]; 3]) -> Option<(char, Vec<(usize, usize)>)> {
    for &player in &['X', 'O'] {
        // Rows & columns
        for i in 0..3 {
            if board[i][0] == player && board[i][1] == player && board[i][2] == player {
                return Some((player, vec![(i, 0), (i, 1), (i, 2)]));
            }
            if board[0][i] == player && board[1][i] == player && board[2][i] == player {
                return Some((player, vec![(0, i), (1, i), (2, i)]));
            }
        }
        // Diagonals
        if board[0][0] == player && board[1][1] == player && board[2][2] == player {
            return Some((player, vec![(0, 0), (1, 1), (2, 2)]));
        }
        if board[0][2] == player && board[1][1] == player && board[2][0] == player {
            return Some((player, vec![(0, 2), (1, 1), (2, 0)]));
        }
    }
    None
}

fn is_board_full(board: &[[char; 3]; 3]) -> bool {
    board.iter().all(|row| row.iter().all(|&c| c != ' '))
}

fn get_random_move(board: &[[char; 3]; 3]) -> Option<(usize, usize)> {
    let mut empty_cells = Vec::new();
    for i in 0..3 {
        for j in 0..3 {
            if board[i][j] == ' ' {
                empty_cells.push((i, j));
            }
        }
    }
    if empty_cells.is_empty() {
        return None;
    }
    use rand::seq::IndexedRandom;
    let mut rng = rand::rng();
    empty_cells.choose(&mut rng).copied()
}

// Minimax algorithm for unbeatable bot
fn minimax(board: &mut [[char; 3]; 3], depth: i32, is_max: bool) -> i32 {
    if let Some('O') = check_winner_simple(board) { return 10 - depth; }
    if let Some('X') = check_winner_simple(board) { return depth - 10; }
    if is_board_full(board) { return 0; }

    if is_max {
        let mut best = -1000;
        for i in 0..3 {
            for j in 0..3 {
                if board[i][j] == ' ' {
                    board[i][j] = 'O';
                    best = best.max(minimax(board, depth + 1, !is_max));
                    board[i][j] = ' ';
                }
            }
        }
        best
    } else {
        let mut best = 1000;
        for i in 0..3 {
            for j in 0..3 {
                if board[i][j] == ' ' {
                    board[i][j] = 'X';
                    best = best.min(minimax(board, depth + 1, !is_max));
                    board[i][j] = ' ';
                }
            }
        }
        best
    }
}

fn get_best_move(board: &[[char; 3]; 3]) -> Option<(usize, usize)> {
    let mut best_val = -1000;
    let mut best_move = None;
    let mut b = *board;

    for i in 0..3 {
        for j in 0..3 {
            if b[i][j] == ' ' {
                b[i][j] = 'O';
                let move_val = minimax(&mut b, 0, false);
                b[i][j] = ' ';
                if move_val > best_val {
                    best_move = Some((i, j));
                    best_val = move_val;
                }
            }
        }
    }
    // Fallback if no best move found (shouldn't happen on non-terminal boards)
    best_move.or_else(|| get_random_move(board))
}
