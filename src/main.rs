use axum::{routing::get, Router};

mod day0;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(day0::hello_bird))
        .route("/-1/seek", get(day0::the_word));

    Ok(router.into())
}
