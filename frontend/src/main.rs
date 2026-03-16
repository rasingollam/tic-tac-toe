use std::str::FromStr;

use gloo_net::http::Request;
use js_sys::Math;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
// Note: `JsCast` methods are available via the web-sys bindings where needed.
use web_sys::window;
use yew::prelude::*;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Difficulty {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Easy" => Ok(Difficulty::Easy),
            "Medium" => Ok(Difficulty::Medium),
            "Hard" => Ok(Difficulty::Hard),
            _ => Err(()),
        }
    }
}

#[derive(Serialize)]
pub struct Move {
    pub session_id: String,
    pub row: usize,
    pub col: usize,
}

#[derive(Serialize)]
pub struct SetDifficultyPayload {
    pub session_id: String,
    pub difficulty: Difficulty,
}

#[derive(Serialize)]
pub struct ResetPayload {
    pub session_id: String,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
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

#[derive(Properties, PartialEq, Default)]
pub struct AppProps {}

pub struct App {
    session_id: String,
    board_data: Option<BoardResponse>,
    is_waiting: bool,
}

pub enum Msg {
    SetData(BoardResponse),
    MakeMove(usize, usize),
    ChangeDifficulty(Difficulty),
    ResetGame,
    SetWaiting(bool),
}

impl Component for App {
    type Message = Msg;
    type Properties = AppProps;

    fn create(ctx: &Context<Self>) -> Self {
        let win = window().unwrap();
        let search = win.location().search().unwrap_or_default();
        let query = web_sys::UrlSearchParams::new_with_str(&search).unwrap();
        
        let session_id = match query.get("session_id") {
            Some(id) => id,
            None => {
                // Generate simple random session ID
                let rand_val = Math::random().to_string();
                // `0.<hex>` -> take slice after "0."
                let new_id = rand_val.get(2..10).unwrap_or(&rand_val).to_string();
                // Push to URL
                let history = win.history().unwrap();
                let new_url = format!("?session_id={}", new_id);
                let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&new_url));
                new_id
            }
        };

        let app = Self { session_id: session_id.clone(), board_data: None, is_waiting: false };

        // Fetch initial board state (use a 'static future that owns needed data)
        let sid = session_id.clone();
        ctx.link().send_future(async move { App::fetch_board(sid).await });

        app
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetData(data) => {
                self.board_data = Some(data);
                self.is_waiting = false;
                true
            }
            Msg::SetWaiting(w) => {
                self.is_waiting = w;
                true
            }
            Msg::MakeMove(r, c) => {
                if self.is_waiting { return false; }
                self.is_waiting = true;
                let sid = self.session_id.clone();
                ctx.link().send_future(async move { App::post_move(sid, r, c).await });
                true
            }
            Msg::ChangeDifficulty(diff) => {
                // Fire-and-forget difficulty change; server will respond with new board
                if self.is_waiting { return false; }
                self.is_waiting = true;
                let sid = self.session_id.clone();
                ctx.link().send_future(async move { App::post_difficulty(sid, diff).await });
                false
            }
            Msg::ResetGame => {
                if self.is_waiting { return false; }
                self.is_waiting = true;
                let sid = self.session_id.clone();
                ctx.link().send_future(async move { App::post_reset(sid).await });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let data = match &self.board_data {
            Some(d) => d,
            None => return html! { <div class="container"><h2>{"Loading..."}</h2></div> },
        };

        // Pre-compute win hash map for easy cell rendering
        let mut win_map = [[false; 3]; 3];
        if let Some(ref wl) = data.win_line {
            for &(r, c) in wl {
                if r < 3 && c < 3 {
                    win_map[r][c] = true;
                }
            }
        }

        let link = ctx.link();

        html! {
            <div class="container">
                <h2>{"Tic-Tac-Toe"}</h2>
                
                <div class="game-settings">
                    <div class="session-info">{ format!("Session ID: {}", self.session_id) }</div>
                    <select
                        class="difficulty-select"
                        onchange={link.callback(|e: Event| {
                            let target: web_sys::HtmlSelectElement = e.target_unchecked_into();
                            let val = target.value();
                            match Difficulty::from_str(&val) {
                                Ok(diff) => Msg::ChangeDifficulty(diff),
                                Err(_) => Msg::SetWaiting(false),
                            }
                        })}
                    >
                        <option value="Easy" selected={data.difficulty == Difficulty::Easy}>{"Bot: Easy"}</option>
                        <option value="Medium" selected={data.difficulty == Difficulty::Medium}>{"Bot: Medium"}</option>
                        <option value="Hard" selected={data.difficulty == Difficulty::Hard}>{"Bot: Hard"}</option>
                    </select>
                </div>

                <div class="scoreboard">
                    <div class="score-box">
                        <span class="score-label">{"Player (X)"}</span>
                        <span class="score-value" style="color: var(--accent-x)">{ data.score_player }</span>
                    </div>
                    <div class="score-box">
                        <span class="score-label">{"Draws"}</span>
                        <span class="score-value" style="color: var(--text-secondary)">{ data.score_draws }</span>
                    </div>
                    <div class="score-box">
                        <span class="score-label">{"Bot (O)"}</span>
                        <span class="score-value" style="color: var(--accent-o)">{ data.score_bot }</span>
                    </div>
                </div>

                <table id="board">
                    <tbody>
                        { for data.board.iter().enumerate().map(|(r, row)| html! {
                            <tr>
                                { for row.iter().enumerate().map(|(c, &cell)| {
                                    let is_win_cell = win_map[r][c];
                                    let mut classes = classes!();
                                    
                                    if cell == 'X' { classes.push("cell-x"); }
                                    else if cell == 'O' { classes.push("cell-o"); }
                                    
                                    if is_win_cell { classes.push("cell-win"); }
                                    
                                    let mut clickable = false;
                                    if data.winner.is_none() && cell == ' ' && !self.is_waiting {
                                        clickable = true;
                                    } else {
                                        classes.push("disabled");
                                    }

                                    let onclick = if clickable {
                                        link.callback(move |_| Msg::MakeMove(r, c))
                                    } else {
                                        // no-op
                                        link.callback(|_| Msg::SetWaiting(false))
                                    };
                                    
                                    // Set inline cursor style for correctness matching vanilla css disabled class structure
                                    let cursor = if clickable { "pointer" } else if cell != ' ' { "default" } else { "not-allowed" };

                                    html! {
                                        <td class={classes} onclick={onclick} style={format!("cursor: {}", cursor)}>
                                            if cell != ' ' {
                                                <span class="animated">{ cell }</span>
                                            }
                                        </td>
                                    }
                                })}
                            </tr>
                        })}
                    </tbody>
                </table>

                <div class="status-container">
                    if self.is_waiting {
                        <p class="status-text"><span class="thinking">{"Bot is thinking..."}</span></p>
                    } else if let Some(winner) = data.winner {
                        <p class="status-text">
                            {"Winner: "}
                            <span style={if winner == 'X' { "color: var(--accent-x)" } else { "color: var(--accent-o)" }}>
                                { winner }{ "!" }
                            </span>
                        </p>
                        <button class="animated" onclick={link.callback(|_| Msg::ResetGame)}>{"Restart Game"}</button>
                    } else if Self::is_draw(&data.board) {
                        <p class="status-text">{"It's a Draw!"}</p>
                        <button class="animated" onclick={link.callback(|_| Msg::ResetGame)}>{"Restart Game"}</button>
                    } else {
                        <p class="status-text">{"Your turn!"}</p>
                        <button onclick={link.callback(|_| Msg::ResetGame)}>{"Restart Game"}</button>
                    }
                </div>
            </div>
        }
    }
}

// Logic implementations for fetching
impl App {
    fn is_draw(board: &[[char; 3]; 3]) -> bool {
        board.iter().all(|row| row.iter().all(|&c| c != ' '))
    }
    // All network helpers are async functions that take owned data so created futures are 'static
    async fn fetch_board(session_id: String) -> Msg {
        let url = format!("http://localhost:3000/board?session_id={}", session_id);
        if let Ok(resp) = Request::get(&url).send().await {
            if let Ok(data) = resp.json::<BoardResponse>().await {
                return Msg::SetData(data);
            }
        }
        Msg::SetWaiting(false)
    }

    async fn post_move(session_id: String, row: usize, col: usize) -> Msg {
        let payload = Move { session_id, row, col };
        let url = "http://localhost:3000/move".to_string();
        let body = serde_json::to_string(&payload).unwrap_or_default();
        if let Ok(resp) = Request::post(&url).header("Content-Type", "application/json").body(&body).expect("build request").send().await {
            if let Ok(data) = resp.json::<BoardResponse>().await {
                return Msg::SetData(data);
            }
        }
        Msg::SetWaiting(false)
    }

    async fn post_difficulty(session_id: String, diff: Difficulty) -> Msg {
        let payload = SetDifficultyPayload { session_id, difficulty: diff };
        let url = "http://localhost:3000/difficulty".to_string();
        let body = serde_json::to_string(&payload).unwrap_or_default();
        if let Ok(resp) = Request::post(&url).header("Content-Type", "application/json").body(&body).expect("build request").send().await {
            if let Ok(data) = resp.json::<BoardResponse>().await {
                return Msg::SetData(data);
            }
        }
        Msg::SetWaiting(false)
    }

    async fn post_reset(session_id: String) -> Msg {
        let payload = ResetPayload { session_id };
        let url = "http://localhost:3000/reset".to_string();
        let body = serde_json::to_string(&payload).unwrap_or_default();
        if let Ok(resp) = Request::post(&url).header("Content-Type", "application/json").body(&body).expect("build request").send().await {
            if let Ok(data) = resp.json::<BoardResponse>().await {
                return Msg::SetData(data);
            }
        }
        Msg::SetWaiting(false)
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
