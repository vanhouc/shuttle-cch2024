use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use game::{GameBoard, GamePiece};

mod game;

type SharedState = Arc<Mutex<GameBoard>>;

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/board", get(board))
        .route("/place/:team/:column", post(place))
        .route("/random-board", get(randomize))
        .route("/reset", post(reset))
        .with_state(Arc::new(Mutex::new(GameBoard::default())))
}

async fn board(State(state): State<SharedState>) -> String {
    state.lock().unwrap().to_string()
}

async fn place(
    Path((team, column)): Path<(GamePiece, u8)>,
    State(state): State<SharedState>,
) -> Response {
    if !(1..5).contains(&column) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let mut state = state.lock().unwrap();
    if let game::GameState::Running = state.state {
        if state.place(team, (column - 1) as usize).is_ok() {
            state.to_string().into_response()
        } else {
            (StatusCode::SERVICE_UNAVAILABLE, state.to_string()).into_response()
        }
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, state.to_string()).into_response()
    }
}

async fn randomize(State(state): State<SharedState>) -> String {
    let mut state = state.lock().unwrap();
    state.randomize();
    state.to_string()
}

async fn reset(State(state): State<SharedState>) -> String {
    let mut state = state.lock().unwrap();
    *state = GameBoard::default();
    state.to_string()
}
