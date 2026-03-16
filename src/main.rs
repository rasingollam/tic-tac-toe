use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct GameState {
    board: [[char; 3]; 3],
    turn: char,
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

#[derive(Serialize)]
struct BoardResponse {
    board: [[char; 3]; 3],
    turn: char,
    winner: Option<char>,
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
        .layer(cors)
        .with_state(state);

    println!("Tic-Tac-Toe API running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_board(
    State(state): State<AppState>,
    Query(query): Query<SessionQuery>,
) -> Json<BoardResponse> {
    let mut games = state.games.lock().await;
    let game = games.entry(query.session_id.clone()).or_insert(GameState {
        board: [[' '; 3]; 3],
        turn: 'X',
    });

    Json(BoardResponse {
        board: game.board,
        turn: game.turn,
        winner: check_winner(&game.board),
    })
}

async fn make_move(
    State(state): State<AppState>,
    Json(mv): Json<Move>,
) -> Json<BoardResponse> {
    let mut games = state.games.lock().await;
    let game = games.entry(mv.session_id.clone()).or_insert(GameState {
        board: [[' '; 3]; 3],
        turn: 'X',
    });

    // Prevent move if out of bounds, cell occupied, or game already won
    if mv.row > 2 || mv.col > 2 || game.board[mv.row][mv.col] != ' ' || check_winner(&game.board).is_some() || game.turn != 'X' {
        return Json(BoardResponse {
            board: game.board,
            turn: game.turn,
            winner: check_winner(&game.board),
        });
    }

    // Player 1 (X) move
    game.board[mv.row][mv.col] = 'X';
    
    // Check if player 1 won or board is full before making bot move
    if check_winner(&game.board).is_some() || is_board_full(&game.board) {
        return Json(BoardResponse {
            board: game.board,
            turn: 'X',
            winner: check_winner(&game.board),
        });
    }

    game.turn = 'O';

    // Bot move (O)
    let mut empty_cells = Vec::new();
    for i in 0..3 {
        for j in 0..3 {
            if game.board[i][j] == ' ' {
                empty_cells.push((i, j));
            }
        }
    }

    if !empty_cells.is_empty() {
        use rand::seq::IndexedRandom;
        let mut rng = rand::rng();
        if let Some(&(r, c)) = empty_cells.choose(&mut rng) {
            game.board[r][c] = 'O';
        }
        game.turn = 'X';
    }

    Json(BoardResponse {
        board: game.board,
        turn: game.turn,
        winner: check_winner(&game.board),
    })
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
    let game = games.entry(payload.session_id.clone()).or_insert(GameState {
        board: [[' '; 3]; 3],
        turn: 'X',
    });
    
    game.board = [[' '; 3]; 3];
    game.turn = 'X';
    
    Json(BoardResponse {
        board: game.board,
        turn: game.turn,
        winner: None,
    })
}

// Check winner
fn check_winner(board: &[[char; 3]; 3]) -> Option<char> {
    for &player in &['X', 'O'] {
        // Rows & columns
        for i in 0..3 {
            if board[i][0] == player && board[i][1] == player && board[i][2] == player {
                return Some(player);
            }
            if board[0][i] == player && board[1][i] == player && board[2][i] == player {
                return Some(player);
            }
        }
        // Diagonals
        if board[0][0] == player && board[1][1] == player && board[2][2] == player {
            return Some(player);
        }
        if board[0][2] == player && board[1][1] == player && board[2][0] == player {
            return Some(player);
        }
    }
    None
}

// Check if board is full
fn is_board_full(board: &[[char; 3]; 3]) -> bool {
    for i in 0..3 {
        for j in 0..3 {
            if board[i][j] == ' ' {
                return false;
            }
        }
    }
    true
}
