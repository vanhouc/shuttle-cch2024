use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Result,
    routing::{delete, get, post, put},
    Json,
};
use chrono::DateTime;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{types::uuid, PgPool};
use tracing::debug;
use uuid::Uuid;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i32,
}

#[derive(Deserialize, Serialize, Debug)]
struct DraftQuote {
    author: String,
    quote: String,
}

#[derive(Deserialize, Serialize)]
struct QuoteList {
    quotes: Vec<Quote>,
    page: i32,
    next_token: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct QuoteListQuery {
    token: String,
}

struct Cursor {
    token: String,
    page: i32,
    created_at: DateTime<chrono::Utc>,
}

pub fn router(pool: PgPool) -> axum::Router {
    axum::Router::new()
        .route("/reset", post(reset))
        .route("/cite/:id", get(cite))
        .route("/remove/:id", delete(remove))
        .route("/undo/:id", put(undo))
        .route("/draft", post(draft))
        .route("/list", get(list))
        .with_state(pool)
}

async fn reset(State(state): State<PgPool>) -> Result<StatusCode> {
    sqlx::query!("DELETE FROM quotes")
        .execute(&state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sqlx::query!("DELETE FROM cursors")
        .execute(&state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

async fn cite(
    Path(id): Path<Uuid>,
    State(state): State<PgPool>,
) -> Result<Json<Quote>, StatusCode> {
    let quote = sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = $1", id)
        .fetch_one(&state)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(quote))
}

async fn remove(
    Path(id): Path<Uuid>,
    State(state): State<PgPool>,
) -> Result<Json<Quote>, StatusCode> {
    let quote = sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = $1", id)
        .fetch_one(&state)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    sqlx::query!("DELETE FROM quotes WHERE id = $1", id)
        .execute(&state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(quote))
}

async fn undo(
    Path(id): Path<Uuid>,
    State(state): State<PgPool>,
    Json(update): Json<DraftQuote>,
) -> Result<Json<Quote>, StatusCode> {
    // Fetch the quote
    let mut quote = sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = $1", id)
        .fetch_one(&state)
        .await
        .map_err(|error| {
            if let sqlx::Error::RowNotFound = error {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Update the quote struct
    quote.author = update.author;
    quote.quote = update.quote;
    quote.version += 1;

    // Update the quote in the database
    sqlx::query!(
        "UPDATE quotes 
         SET author = $1, quote = $2, version = $3
         WHERE id = $4",
        quote.author,
        quote.quote,
        quote.version,
        quote.id,
    )
    .execute(&state)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(quote))
}

async fn draft(
    State(state): State<PgPool>,
    Json(draft): Json<DraftQuote>,
) -> Result<(StatusCode, Json<Quote>), StatusCode> {
    let id = Uuid::new_v4();
    // Insert the quote into the database
    sqlx::query!(
        "INSERT INTO quotes (id, author, quote)
         VALUES ($1, $2, $3)",
        id,
        draft.author,
        draft.quote,
    )
    .execute(&state)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    debug!("inserted quote with id: {:?}", id);

    // Return the quote
    cite(Path(id), State(state))
        .await
        .map(|quote| (StatusCode::CREATED, quote))
}

async fn list(
    State(state): State<PgPool>,
    query: Option<Query<QuoteListQuery>>,
) -> Result<Json<QuoteList>, StatusCode> {
    match query {
        Some(Query(QuoteListQuery { token })) => list_with_token(token, state).await,
        None => list_new(state).await,
    }
}

async fn list_new(state: PgPool) -> Result<Json<QuoteList>, StatusCode> {
    let mut quotes = sqlx::query_as!(
        Quote,
        "SELECT * FROM quotes ORDER BY created_at ASC LIMIT 4"
    )
    .fetch_all(&state)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let last_quote = quotes.get(3);

    let next_token = match last_quote {
        Some(quote) => {
            let cursor = Cursor {
                token: generate_random_ascii_string(16),
                page: 1,
                created_at: quote.created_at,
            };
            sqlx::query!(
                "INSERT INTO cursors (token, created_at) VALUES ($1, $2)",
                cursor.token,
                cursor.created_at
            )
            .execute(&state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Some(cursor.token)
        }
        None => None,
    };

    // Remove the last quote from the list if we got 4 entries
    if last_quote.is_some() {
        quotes.pop();
    }

    let list = QuoteList {
        quotes,
        page: 1,
        next_token,
    };

    Ok(Json(list))
}

async fn list_with_token(token: String, state: PgPool) -> Result<Json<QuoteList>, StatusCode> {
    let cursor = sqlx::query_as!(
        Cursor,
        "SELECT token, page, created_at FROM cursors WHERE token = $1",
        token
    )
    .fetch_optional(&state)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::BAD_REQUEST)?;

    let page = cursor.page;

    let quotes = sqlx::query_as!(
        Quote,
        "SELECT * FROM quotes ORDER BY created_at ASC OFFSET $1 * 3 LIMIT 4",
        page,
    )
    .fetch_all(&state)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let last_quote = quotes.get(3);

    let next_token = match last_quote {
        Some(_) => {
            sqlx::query!(
                "UPDATE cursors SET page = $1 WHERE token = $2",
                cursor.page + 1,
                cursor.token
            )
            .execute(&state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Some(cursor.token)
        }
        None => {
            sqlx::query!("DELETE FROM cursors WHERE token = $1", cursor.token)
                .execute(&state)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            None
        }
    };

    let list = QuoteList {
        quotes,
        page: page + 1,
        next_token,
    };

    Ok(Json(list))
}

fn generate_random_ascii_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{body::Body, extract::Request, http::header::CONTENT_TYPE};
    use http_body_util::BodyExt;
    use tower::{Service, ServiceExt};

    #[sqlx::test]
    async fn test_draft(pool: PgPool) {
        let app = router(pool.clone());

        let draft = DraftQuote {
            author: "FOO".to_string(),
            quote: "BAR".to_string(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/draft")
                    .header(CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_vec(&draft).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let db_quote = sqlx::query_as!(Quote, "SELECT * FROM quotes")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let quote: Quote = serde_json::from_slice(&body).unwrap();

        assert_eq!(db_quote, quote);
    }

    #[sqlx::test(fixtures("quotes_3"))]
    async fn test_list(pool: PgPool) {
        let quotes = get_quotes(&pool).await;
        let app = router(pool);

        let response = app
            .oneshot(Request::builder().uri("/list").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let list: QuoteList = serde_json::from_slice(&body).unwrap();

        assert_eq!(quotes, list.quotes);
        assert_eq!(1, list.page);
        assert!(list.next_token.is_none());
    }

    #[sqlx::test(fixtures("quotes_4"))]
    async fn test_list_token(pool: PgPool) {
        let quotes = get_quotes(&pool).await;
        let mut app = router(pool);

        let response = app
            .call(Request::builder().uri("/list").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let list: QuoteList = serde_json::from_slice(&body).unwrap();

        // Assert we get the first 3 values but not the 4th
        assert_eq!(&quotes[..3], &list.quotes);

        let token = list.next_token.unwrap();

        let response = app
            .call(
                Request::builder()
                    .uri(format!("/list?token={token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let list: QuoteList = serde_json::from_slice(&body).unwrap();

        // Assert we only got the 4th value
        assert_eq!(&quotes[3..], &list.quotes);
        assert_eq!(2, list.page);
        assert_eq!(None, list.next_token);
    }

    #[sqlx::test]
    async fn test_quote_order(pool: PgPool) {
        let mut app = router(pool);

        for i in 1..6 {
            let draft = DraftQuote {
                author: "FOO".to_string(),
                quote: i.to_string(),
            };
            app.call(
                Request::builder()
                    .method("POST")
                    .uri("/draft")
                    .header(CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_vec(&draft).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        }

        let response = app
            .call(Request::builder().uri("/list").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let list: QuoteList = serde_json::from_slice(&body).unwrap();

        let ids: Vec<i32> = list
            .quotes
            .iter()
            .map(|quote| quote.quote.parse().unwrap())
            .collect();

        let token = list.next_token.unwrap();

        assert_eq!(vec![1, 2, 3], ids);

        let response = app
            .call(
                Request::builder()
                    .uri(format!("/list?token={token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let list: QuoteList = serde_json::from_slice(&body).unwrap();

        let ids: Vec<i32> = list
            .quotes
            .iter()
            .map(|quote| quote.quote.parse().unwrap())
            .collect();

        assert_eq!(vec![4, 5], ids);
        assert!(list.next_token.is_none());
    }

    async fn get_quotes(pool: &PgPool) -> Vec<Quote> {
        sqlx::query_as!(Quote, "SELECT * FROM quotes ORDER BY created_at ASC")
            .fetch_all(pool)
            .await
            .unwrap()
    }
}
