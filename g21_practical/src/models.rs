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

// Define the `Stock` struct to represent stock information in the application.
// Like `User`, this struct also derives `Serialize`, `Deserialize`, and `Debug` traits.
#[derive(Serialize, Deserialize, Debug)]
pub struct Stock {
    pub id: Uuid,         // Unique identifier for the stock.
    pub symbol: String,   // Symbol representing the stock, like "AAPL" for Apple.
    pub price: f64,       // Current price of the stock.
}

// Define the `Transaction` struct to represent a stock transaction in the application.
// This struct is also serializable and deserializable, allowing it to be used with various data formats.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub id: Uuid,                 // Unique identifier for the transaction.
    pub user_id: Uuid,            // The ID of the user making the transaction.
    pub stock_id: Uuid,           // The ID of the stock involved in the transaction.
    pub quantity: i32,            // The number of stock units involved in the transaction.
    pub transaction_type: String, // Type of transaction, either "buy" or "sell".
}