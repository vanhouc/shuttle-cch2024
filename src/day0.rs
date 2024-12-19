use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};

pub async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

pub async fn the_word() -> impl IntoResponse {
    (
        StatusCode::FOUND,
        [(
            header::LOCATION,
            "https://www.youtube.com/watch?v=9Gc4QTqslN4",
        )],
    )
}
