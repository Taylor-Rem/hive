use axum::extract::State;
use axum::http::StatusCode;
use crate::state::app_state::AppState;
use crate::jobs::orchestrators::schema::populate_schema::populate_schema;

pub async fn populate_schema_handler(
    State(state): State<AppState>,
) -> Result<&'static str, StatusCode> {
    populate_schema(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok("Populate schema route hit successfully")
}
