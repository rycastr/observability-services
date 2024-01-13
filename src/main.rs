use std::env;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db_pool: Pool<Postgres>,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    tracing_subscriber::fmt().with_target(true).json().init();

    let db_url = env::var("DATABASE_URL").unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let app_state = AppState { db_pool: pool };
    let app = Router::new()
        .route("/api/todos", post(create_todo))
        .route("/api/todos/:todo_id", patch(update_todo_status))
        .route("/api/todos", get(fetch_todos))
        .with_state(app_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    info!("Listening on port 8000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Todo {
    id: Uuid,
    title: String,
    status: bool,
}

#[derive(Serialize, Deserialize)]
struct CreateTodoReq {
    title: String,
}

async fn create_todo(
    State(app_state): State<AppState>,
    Json(create_todo_req): Json<CreateTodoReq>,
) -> Response {
    let result = sqlx::query!(
        r#"INSERT INTO todos (id, title, status) values ($1, $2, $3)"#,
        Uuid::new_v4(),
        create_todo_req.title,
        false
    )
    .execute(&app_state.db_pool)
    .await;

    match result {
        Ok(q_result) => {
            if q_result.rows_affected() > 1 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"message": "Unable to create todo"})),
                )
                    .into_response();
            }
            (StatusCode::CREATED, Json(json!({"ok": true}))).into_response()
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "Unable to create todo"})),
        )
            .into_response(),
    }
}

#[derive(Serialize, Deserialize)]
struct UpdateTodoStatusReq {
    status: bool,
}

async fn update_todo_status(
    State(app_state): State<AppState>,
    Path(todo_id): Path<Uuid>,
    Json(update_todo_status_req): Json<UpdateTodoStatusReq>,
) -> Response {
    let result = sqlx::query!(
        r#"UPDATE todos SET status = $1 WHERE id = $2"#,
        update_todo_status_req.status,
        todo_id
    )
    .execute(&app_state.db_pool)
    .await;

    match result {
        Ok(_) => (StatusCode::OK, Json(json!({"ok": true}))).into_response(),
        Err(_) => (StatusCode::BAD_REQUEST, "Unable to update todo status").into_response(),
    }
}

async fn fetch_todos(State(app_state): State<AppState>) -> Response {
    let result = sqlx::query_as!(Todo, r#"SELECT id, title, status FROM todos"#)
        .fetch_all(&app_state.db_pool)
        .await;

    match result {
        Ok(todos) => (StatusCode::OK, Json(todos)).into_response(),
        Err(_) => (StatusCode::BAD_REQUEST, "Unable to fetch todos").into_response(),
    }
}
