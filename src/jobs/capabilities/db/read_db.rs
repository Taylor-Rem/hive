use crate::db::connect::DbState;
use std::collections::HashMap;
use anyhow::Result;
use sqlx::Row;

#[derive(Debug)]
pub struct DbSchema {
    pub tables: HashMap<String, DbTable>,
}

#[derive(Debug)]
pub struct DbTable {
    pub columns: Vec<DbColumn>,
    pub foreign_keys: Vec<ForeignKey>,
    pub indexes: Vec<DbIndex>
}
#[derive(Debug)]
pub struct DbColumn {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default: Option<String>,
}
#[derive(Debug)]
pub struct ForeignKey {
    pub column: String,
    pub referenced_table: String,
    pub referenced_column: bool,
}
#[derive(Debug)]
pub struct DbIndex {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default: Option<String>,
}

pub async fn read_db_schema(db: &DbState) -> Result<DbSchema> {
    let rows = sqlx::query(
        r#"
        SELECT
            table_name,
            column_name,
            data_type,
            is_nullable,
            column_default
        FROM information_schema.columns
        WHERE table_schema = 'public'
        ORDER BY table_name, ordinal_position
        "#
    )
    .fetch_all(&db.pool)
    .await?;

    let mut tables: HashMap<String, DbTable> = HashMap::new();

    for row in rows {
        let table_name: String = row.get("table_name");

        let table = tables
            .entry(table_name)
            .or_insert_with(|| DbTable {
                columns: Vec::new(),
                foreign_keys: Vec::new(),
                indexes: Vec::new()
            });

        table.columns.push(DbColumn {
            name: row.get("column_name"),
            data_type: row.get("data_type"),
            is_nullable: row.get::<String, _>("is_nullable") == "YES",
            default: row.get("column_default"),
        });
    }

    Ok(DbSchema { tables })
}
