use anyhow::Result;
use axum::{
    extract::Extension, http::StatusCode, response::Json, routing::get,
    AddExtensionLayer, Router,
};
use import::get_bg_names;
use log::info;
use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use std::result::Result as StdResult;

mod import;

const CARGO_PKG_VERSION: Option<&'static str> =
    option_env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let pool = SqlitePool::connect("db.sqlite").await?;

    let app = Router::new()
        .route("/version", get(version))
        .route("/bgs", get(list_bgs_route))
        .route("/import", get(import_bgs_route))
        .layer(AddExtensionLayer::new(pool));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn version() -> &'static str {
    match CARGO_PKG_VERSION {
        Some(version) => version,
        None => "unknown",
    }
}

async fn import_bgs_route(
    Extension(pool): Extension<SqlitePool>,
) -> Result<String, ErrorDto> {
    let result = import_bgs(pool).await;
    match result {
        Ok(number_added) => {
            info!("Added {} bgs", number_added);
            Ok(format!("Added {} bgs", number_added))
        }
        Err(err) => Err(error_into_response(err)),
    }
}

async fn import_bgs(pool: SqlitePool) -> Result<usize> {
    let mut number_added = 0;

    let bg_names = get_bg_names().await?;
    for bg_name in bg_names {
        let bg = get_bg_by_name(&bg_name, pool.clone()).await?;
        match bg {
            Some(_) => (), // Don't import it, because we already have it
            None => {
                add_bg(&bg_name, pool.clone()).await?;
                number_added += 1;
            }
        }
    }
    Ok(number_added)
}

async fn get_bg_by_name(name: &str, pool: SqlitePool) -> Result<Option<Bg>> {
    let bg = sqlx::query_as!(Bg, "SELECT * FROM bgs WHERE name = ?;", name)
        .fetch_optional(&pool)
        .await?;
    Ok(bg)
}

async fn add_bg(name: &str, pool: SqlitePool) -> Result<()> {
    sqlx::query_as!(Bg, "INSERT INTO bgs (name) VALUES (?);", name)
        .execute(&pool)
        .await?;
    Ok(())
}

#[axum_debug::debug_handler]
async fn list_bgs_route(
    Extension(pool): Extension<SqlitePool>,
) -> StdResult<Json<BgsDto>, ErrorDto> {
    match list_bgs(pool).await {
        Ok(bgs) => Ok(Json(BgsDto { bgs })),
        Err(err) => Err(error_into_response(err)),
    }
}

async fn list_bgs(pool: SqlitePool) -> Result<Vec<Bg>> {
    let bgs = sqlx::query_as!(Bg, "SELECT * FROM bgs;")
        .fetch_all(&pool)
        .await?;
    Ok(bgs)
}

fn error_into_response(err: anyhow::Error) -> ErrorDto {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct Bg {
    id: i64,
    name: String,
}

#[derive(Debug, Serialize)]
struct BgsDto {
    bgs: Vec<Bg>,
}

type ErrorDto = (StatusCode, String);
