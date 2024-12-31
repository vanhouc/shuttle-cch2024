use axum::{
    routing::{get, post},
    Router,
};

mod day0;
mod day12;
mod day2;
mod day5;
mod day9;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(day0::hello_bird))
        .route("/-1/seek", get(day0::the_word))
        .route("/2/dest", get(day2::dest))
        .route("/2/key", get(day2::key))
        .route("/2/v6/dest", get(day2::dest_v6))
        .route("/2/v6/key", get(day2::key_v6))
        .route("/5/manifest", post(day5::manifest))
        .nest("/9", day9::router())
        .nest("/12", day12::router());

    Ok(router.into())
}
