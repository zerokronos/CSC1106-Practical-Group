// Import necessary traits from the `serde` crate to enable serialization and deserialization of data structures.
// `serde` is commonly used for converting data structures to and from formats like JSON.
// Import `uuid` for generating and handling universally unique identifiers (UUIDs).
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Define the `User` struct to represent a user in the application.
// This struct derives `Serialize` and `Deserialize` so it can be easily converted to and from JSON and other data formats.
// It also derives `Debug` to allow for formatted output, useful for debugging.
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: Uuid,              // Unique identifier for the user.
    pub username: String,      // Username of the user.
    pub hashed_password: String, // Password of the user stored in a hashed form for security.
}

#[derive(Deserialize)]
pub struct Login {
    pub username: String, // Username of the user
    pub password: String, // Pasword of the user that key it in
}

// Define the `BugReport` struct to represent a bug report.
#[derive(Serialize, Deserialize, Debug)]
pub struct BugReport {
    pub id: Uuid,                 // Unique identifier of the bug.
    pub reported_by: Uuid,       // The ID of the user that reported the bug.
    pub fixed_by: Uuid,         // The ID of the user that fixed the bug.
    pub title: String,          // The title of the bug
    pub description: String,   // The description of the bug
    pub serverity: String,    // The severity of the bug
}

// Define the 'CreateBug' struct to represent the creation of a bug
#[derive(Deserialize)]
pub struct CreateBug {
    pub reported_by: String, // The username of the person that reported the bug
    pub title: String,    // The title of the bug
    pub description: String, // The description of the bug
    pub serverity: String, // The severity of the bug
}

