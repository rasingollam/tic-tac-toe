use crate::models::{Difficulty, GameState};
use rand::seq::IndexedRandom;
use rand::RngExt;

pub fn check_winner_simple(board: &[[char; 3]; 3]) -> Option<char> {
    check_winner_detailed(board).map(|(w, _)| w)
}

pub fn check_winner_detailed(board: &[[char; 3]; 3]) -> Option<(char, Vec<(usize, usize)>)> {
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

pub fn is_board_full(board: &[[char; 3]; 3]) -> bool {
    board.iter().all(|row| row.iter().all(|&c| c != ' '))
}

pub fn get_random_move(board: &[[char; 3]; 3]) -> Option<(usize, usize)> {
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
    let mut rng = rand::rng();
    empty_cells.choose(&mut rng).copied()
}

// Minimax algorithm for unbeatable bot
fn minimax(board: &mut [[char; 3]; 3], depth: i32, is_max: bool) -> i32 {
    if let Some('O') = check_winner_simple(board) {
        return 10 - depth;
    }
    if let Some('X') = check_winner_simple(board) {
        return depth - 10;
    }
    if is_board_full(board) {
        return 0;
    }

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

pub fn get_best_move(board: &[[char; 3]; 3]) -> Option<(usize, usize)> {
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
    best_move.or_else(|| get_random_move(board))
}

pub fn make_bot_move(game: &mut GameState) {
    let move_pos = match game.difficulty {
        Difficulty::Easy => get_random_move(&game.board),
        Difficulty::Medium => {
            let mut rng = rand::rng();
            if rng.random_bool(0.5) {
                get_random_move(&game.board)
            } else {
                get_best_move(&game.board)
            }
        }
        Difficulty::Hard => get_best_move(&game.board),
    };

    if let Some((r, c)) = move_pos {
        game.board[r][c] = 'O';
    }
}
