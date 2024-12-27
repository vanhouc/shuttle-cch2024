use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    routing::{get, post},
    Router,
};
use leaky_bucket::RateLimiter;

mod day0;
mod day2;
mod day5;
mod day9;

struct AppState {
    rate_limiter: Mutex<RateLimiter>,
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let state = AppState {
        rate_limiter: Mutex::new(
            RateLimiter::builder()
                .max(5)
                .initial(5)
                .interval(Duration::from_secs(1))
                .build(),
        ),
    };

    let router = Router::new()
        .route("/", get(day0::hello_bird))
        .route("/-1/seek", get(day0::the_word))
        .route("/2/dest", get(day2::dest))
        .route("/2/key", get(day2::key))
        .route("/2/v6/dest", get(day2::dest_v6))
        .route("/2/v6/key", get(day2::key_v6))
        .route("/5/manifest", post(day5::manifest))
        .route("/9/milk", post(day9::milk))
        .route("/9/refill", post(day9::refill))
        .with_state(Arc::new(state));

    Ok(router.into())
}
