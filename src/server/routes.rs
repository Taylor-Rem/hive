use axum::{Router, routing::{get}};
use crate::handlers::jobs::{populate_schema_handler};
use crate::state::app_state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/populate_schema", get(populate_schema_handler))
}
