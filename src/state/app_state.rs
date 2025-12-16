use crate::db::connect::DbState;

#[derive(Clone)]
pub struct AppState {
    pub db: DbState,
}
