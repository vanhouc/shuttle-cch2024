use std::{sync::Arc, time::Duration};

use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use leaky_bucket::RateLimiter;
use serde::{Deserialize, Serialize};

use crate::AppState;

pub async fn milk(
    State(state): State<Arc<AppState>>,
    quantity: Result<Json<MilkRequest>, JsonRejection>,
) -> axum::response::Response {
    let rate_limiter = &state.rate_limiter.lock().unwrap();
    if rate_limiter.try_acquire(1) {
        match quantity {
            Ok(Json(MilkRequest::Gallons { gallons })) => Json(MilkRequest::Liters {
                liters: gallons * 3.7854111,
            })
            .into_response(),
            Ok(Json(MilkRequest::Liters { liters })) => Json(MilkRequest::Gallons {
                gallons: liters / 3.7854111,
            })
            .into_response(),
            Ok(Json(MilkRequest::Pints { pints })) => Json(MilkRequest::Litres {
                litres: pints / 1.75975,
            })
            .into_response(),
            Ok(Json(MilkRequest::Litres { litres })) => Json(MilkRequest::Pints {
                pints: litres * 1.75975,
            })
            .into_response(),
            Err(JsonRejection::MissingJsonContentType(_)) => "Milk withdrawn\n".into_response(),
            _ => StatusCode::BAD_REQUEST.into_response(),
        }
    } else {
        (StatusCode::TOO_MANY_REQUESTS, "No milk available\n").into_response()
    }
}

pub async fn refill(State(state): State<Arc<AppState>>) -> axum::response::Response {
    let mut rate_limiter = state.rate_limiter.lock().unwrap();
    *rate_limiter = RateLimiter::builder()
        .max(5)
        .initial(5)
        .interval(Duration::from_secs(1))
        .build();
    StatusCode::OK.into_response()
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum MilkRequest {
    Both { gallons: f64, liters: f64 },
    BothUk { pints: f64, litres: f64 },
    Combo { gallons: f64, pints: f64 },
    BothLiters { liters: f64, litres: f64 },
    Gallons { gallons: f64 },
    Liters { liters: f64 },
    Pints { pints: f64 },
    Litres { litres: f64 },
}
