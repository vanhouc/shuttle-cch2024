use axum::{routing::get, Router};

mod day0;
mod day2;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(day0::hello_bird))
        .route("/-1/seek", get(day0::the_word))
        .route("/2/dest", get(day2::dest))
        .route("/2/key", get(day2::key))
        .route("/2/v6/dest", get(day2::dest_v6))
        .route("/2/v6/key", get(day2::key_v6));

    Ok(router.into())
}
