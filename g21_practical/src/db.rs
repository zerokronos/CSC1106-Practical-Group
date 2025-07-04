// Import necessary items from the `sqlx` crate for SQLite database connection pooling.
// `Pool` is used to manage a pool of database connections, while `Sqlite` and `SqlitePoolOptions` are specific to SQLite.
use sqlx::{Pool, Sqlite};
use sqlx::sqlite::SqlitePoolOptions;

// Define an asynchronous function `init_db` that initializes a connection pool to an SQLite database.
// This function returns a `Pool<Sqlite>` type, which represents a pool of SQLite connections.
pub async fn init_db() -> Pool<Sqlite> {
    // Create a new instance of `SqlitePoolOptions` to configure the connection pool settings.
    SqlitePoolOptions::new()
        .max_connections(5) // Set the maximum number of connections in the pool to 5.
        .connect("sqlite::memory:") // Connect to an in-memory SQLite database.
        .await // Since database connections are asynchronous operations, await the completion.
        .expect("DB connection failed") // Panic with an error message if the connection fails.
}