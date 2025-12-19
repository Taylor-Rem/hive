use anyhow::Result;
use serde::Serialize;
use std::fs;

use crate::jobs::capabilities::db::read_db::{
    DbSchema,
    DbTable,
    DbColumn,
};

#[derive(Serialize)]
struct TomlSchema {
    table: Vec<TomlTable>,
}

#[derive(Serialize)]
struct TomlTable {
    name: String,
    column: Vec<TomlColumn>,
}

#[derive(Serialize)]
struct TomlColumn {
    name: String,
    data_type: String,
    nullable: bool,
    default: Option<String>,
}

pub fn write_schema_toml(schema: DbSchema, path: &str) -> Result<()> {
    let mut tables: Vec<TomlTable> = schema
        .tables
        .into_iter()
        .map(|(name, table)| to_toml_table(name, table))
        .collect();

    tables.sort_by(|a, b| a.name.cmp(&b.name));

    let toml_schema = TomlSchema { table: tables };

    let toml_string = toml::to_string_pretty(&toml_schema)?;
    fs::write(path, toml_string)?;

    Ok(())
}

fn to_toml_table(name: String, table: DbTable) -> TomlTable {
    TomlTable {
        name,
        column: table.columns.into_iter().map(to_toml_column).collect(),
    }
}


fn to_toml_column(col: DbColumn) -> TomlColumn {
    TomlColumn {
        name: col.name,
        data_type: col.data_type,
        nullable: col.is_nullable,
        default: col.default,
    }
}
