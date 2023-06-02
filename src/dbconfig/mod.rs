#[path = "schemas/sqlite/schema.rs"]
pub mod schema;


#[macro_export]
#[cfg(sqlite)]
macro_rules! import_database_connections {
    () => {
        use diesel::SqliteConnection;
    };
}

#[derive(diesel::MultiConnection)]
pub enum AnyConnection {
    Postgresql(diesel::PgConnection),
    Sqlite(diesel::SqliteConnection),
}

