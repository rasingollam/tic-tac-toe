use crate::models::GameState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

#[derive(Clone)]
pub struct AppState {
    pub games: Arc<AsyncMutex<HashMap<String, GameState>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            games: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }
}
