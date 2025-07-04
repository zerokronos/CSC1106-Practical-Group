// Import necessary items from the `sqlx` crate for SQLite database connection pooling.
// `Pool` is used to manage a pool of database connections, while `Sqlite` and `SqlitePoolOptions` are specific to SQLite.
use sqlx::{Pool, Sqlite};
use sqlx::sqlite::SqlitePoolOptions;
use uuid::Uuid;
use std::fs;


// Define an asynchronous function `init_db` that initializes a connection pool to an SQLite database.
// This function returns a `Pool<Sqlite>` type, which represents a pool of SQLite connections.
pub async fn init_db() -> Pool<Sqlite> {
    // Create a new instance of `SqlitePoolOptions` to configure the connection pool settings.
    let pool = SqlitePoolOptions::new()
        .max_connections(5) // Set the maximum number of connections in the pool to 5.
        .connect("sqlite::memory:?cache=shared") // Connect to an in-memory SQLite database.
        .await // Since database connections are asynchronous operations, await the completion.
        .expect("DB connection failed"); // Panic with an error message if the connection fails.

       let sql = fs::read_to_string("migrations/schema.sql").expect("Failed to read schema.sql");
        sqlx::query(&sql)
        .execute(&pool)
        .await
        .expect("Failed to execute schema.sql");

    // Insert admin user
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, username, hashed_password) VALUES (?, ?, ?)")
        .bind(&user_id.as_bytes()[..])
        .bind("admin")
        .bind("fake") // hash in real apps!
        .execute(&pool)
        .await
        .expect("Failed to insert admin user");

    // Insert a normal user
    let user_id2 = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, username, hashed_password) VALUES (?, ?, ?)")
        .bind(&user_id2.as_bytes()[..])
        .bind("normal_user")
        .bind("fake") // hash in real apps!
        .execute(&pool)
        .await
        .expect("Failed to insert normal user");

    // Insert a project record
    let project_id = Uuid::new_v4();
    sqlx::query("INSERT INTO projectRecord (id, user_id, project_name, project_description) VALUES (?, ?, ?, ?)")
        .bind(&project_id.as_bytes()[..])
        .bind(&user_id.as_bytes()[..])
        .bind("Project A")
        .bind("Description of Project A")
        .execute(&pool)
        .await
        .expect("Failed to insert project record");

    let bug_id = Uuid::new_v4();
    // Insert a bug report
    sqlx::query("INSERT INTO bugReport (id, project_id, title, description, reported_by, severity, is_fixed) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&bug_id.as_bytes()[..])
        .bind(&project_id.as_bytes()[..])
        .bind("BUG in Login Feature")
        .bind("There is a bug in the login feature that prevents users from logging in.")
        .bind(&user_id.as_bytes()[..])
        .bind("high")
        .bind(false)
        .execute(&pool)
        .await
        .expect("Failed to insert bug report");

    
    pool
}
