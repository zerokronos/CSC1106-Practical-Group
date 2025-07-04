// Import necessary traits from the `serde` crate to enable serialization and deserialization of data structures.
// `serde` is commonly used for converting data structures to and from formats like JSON.
// Import `uuid` for generating and handling universally unique identifiers (UUIDs).
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;

// Define the `User` struct to represent a user in the application.
// This struct derives `Serialize` and `Deserialize` so it can be easily converted to and from JSON and other data formats.
// It also derives `Debug` to allow for formatted output, useful for debugging.
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct User {
    pub id: Uuid,              // Unique identifier for the user.
    pub username: String,      // Username of the user.
    pub hashed_password: String, // Password of the user stored in a hashed form for security.
}

// Struct for incoming login requests
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// Struct for login responses
#[derive(Serialize)]
pub struct LoginResponse {
    pub status: String,
    pub message: String,
    pub token: Option<String>,
}

// Define the `BugReport` struct to represent a bug report.
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct BugReport {
    pub id: Uuid,                 // Unique identifier of the bug.
    pub project_id: Uuid, // The ID of the project that the bug belongs to.
    pub title: String,          // The title of the bug
    pub description: String,   // The description of the bug
    pub reported_by: Uuid,       // The ID of the user that reported the bug.
    pub fixed_by: Option<Uuid>,   // The ID of the user that fixed the bug.
    pub severity: String,    // The severity of the bug
    pub is_fixed: bool, // Indicates whether the bug has been fixed or not.
    pub created_at: String, // Timestamp of when the bug was created
}

// Define the 'CreateBug' struct to represent the creation of a bug
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct CreateBug {
    pub reported_by: String, // The username of the person that reported the bug
    pub title: String,    // The title of the bug
    pub description: String, // The description of the bug
    pub severity: String, // The severity of the bug
    pub project_name: String, // The name of the project that the bug belongs to
}


#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct ProjectRecord {
    pub id: Uuid,
    pub project_name: String,
    pub project_description: String,
    pub created_at: String, // Timestamp of when the project was created
    pub user_id: Uuid, // The ID of the user that created the project
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BugAssignment { 
    pub bug_id: Uuid, // The ID of the bug being assigned
    pub user_id: Uuid, // The ID of the user to whom the bug is assigned
}

// Simple user struct for dropdowns (no password needed)
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct SimpleUser {
    pub id: Uuid,
    pub username: String,
}