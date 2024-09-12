use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    database: DatabseConfig,
    binding: BindingConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BindingConfig {
    interface: String,
    port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DatabseConfig {
    Sqlite(SqliteConfig),
    Postgres(PostgresConfig),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SqliteConfig {

}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostgresConfig {

}
