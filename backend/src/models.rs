use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Clone)]
pub struct GameState {
    pub board: [[char; 3]; 3],
    pub turn: char,
    pub last_active: Instant,
    pub difficulty: Difficulty,
    pub score_player: u32,
    pub score_bot: u32,
    pub score_draws: u32,
}

impl GameState {
    pub fn new() -> Self {
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

#[derive(Deserialize)]
pub struct SessionQuery {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct Move {
    pub session_id: String,
    pub row: usize,
    pub col: usize,
}

#[derive(Deserialize)]
pub struct SetDifficultyPayload {
    pub session_id: String,
    pub difficulty: Difficulty,
}

#[derive(Deserialize)]
pub struct ResetPayload {
    pub session_id: String,
}

#[derive(Serialize)]
pub struct BoardResponse {
    pub board: [[char; 3]; 3],
    pub turn: char,
    pub winner: Option<char>,
    pub win_line: Option<Vec<(usize, usize)>>,
    pub score_player: u32,
    pub score_bot: u32,
    pub score_draws: u32,
    pub difficulty: Difficulty,
}
