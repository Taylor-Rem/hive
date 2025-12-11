use crate::db::connect::DbState;
use axum::{Router, routing::get};

pub fn create_server(_db_state: DbState) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello from Hive server!" }))
}