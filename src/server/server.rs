use axum::{Router};
use crate::state::app_state::AppState;
use crate::server::routes::routes;


pub fn create_server(state: AppState) -> Router {
    Router::new()
        .merge(routes())
        .with_state(state)
}
