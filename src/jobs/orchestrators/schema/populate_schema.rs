use anyhow::Result;
use crate::state::app_state::AppState;
use crate::jobs::capabilities::db::read_db::read_db_schema;
use crate::jobs::capabilities::schema::write_schema::write_schema_toml;

pub async fn populate_schema(state: &AppState) -> Result<()> {
    let schema = read_db_schema(&state.db).await?;
    write_schema_toml(schema, "schema/schema.toml")?;
    Ok(())
}
