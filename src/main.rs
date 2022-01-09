use anyhow::Result;
use axum::{extract::Extension, routing::get, AddExtensionLayer, Router};
use axum_debug::debug_handler;
use import::get_bg_names;
use serde::Serialize;
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};

mod import;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect("db.sqlite").await?;

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/import", get(import_bgs_route))
        .layer(AddExtensionLayer::new(pool));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[debug_handler]
async fn import_bgs_route(Extension(pool): Extension<Pool<Sqlite>>) -> String {
    let result = import_bgs(pool).await;
    match result {
        Ok(number_added) => {
            log::info!("Added {} bgs", number_added);
            format!("Added {} bgs", number_added)
        }
        Err(err) => format!("{}", err),
    }
}

async fn import_bgs(pool: Pool<Sqlite>) -> Result<usize> {
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

async fn get_bg_by_name(name: &str, pool: Pool<Sqlite>) -> Result<Option<Bg>> {
    let bg = sqlx::query_as!(Bg, "SELECT * FROM bgs WHERE name = ?;", name)
        .fetch_optional(&pool)
        .await?;
    Ok(bg)
}

async fn add_bg(name: &str, pool: Pool<Sqlite>) -> Result<()> {
    sqlx::query_as!(Bg, "INSERT INTO bgs (name) VALUES (?);", name)
        .execute(&pool)
        .await?;
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct Bg {
    id: i64,
    name: String,
}
