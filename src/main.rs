use axum::{
    routing::{get, post},
    Router,
};
use axum_embed::ServeEmbed;
use rust_embed::RustEmbed;

mod day0;
mod day12;
mod day16;
mod day19;
mod day2;
mod day23;
mod day5;
mod day9;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(local_uri = option_env!("DATABASE_URL").unwrap_or(""))]
    pool: sqlx::PgPool,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let router = Router::new()
        .route("/", get(day0::hello_bird))
        .route("/-1/seek", get(day0::the_word))
        .route("/2/dest", get(day2::dest))
        .route("/2/key", get(day2::key))
        .route("/2/v6/dest", get(day2::dest_v6))
        .route("/2/v6/key", get(day2::key_v6))
        .route("/5/manifest", post(day5::manifest))
        .nest("/9", day9::router())
        .nest("/12", day12::router())
        .nest("/16", day16::router())
        .nest("/19", day19::router(pool))
        .nest("/23", day23::router())
        .nest_service("/assets", ServeEmbed::<Assets>::new());

    Ok(router.into())
}

#[derive(RustEmbed, Clone)]
#[folder = "assets/"]
struct Assets;
