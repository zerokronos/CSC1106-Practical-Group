// Import necessary modules and traits from external crates `jsonwebtoken`, `serde`, `std`, and `uuid`.
// These are used for JWT creation/validation, serialization/deserialization, environment variable handling, and UUID generation.
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::env;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};

// Define a struct called `Claims` that will hold the data to be encoded into the JWT.
// This struct derives `Serialize` and `Deserialize` traits to facilitate JSON conversion.
#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String, // `sub` stands for subject and typically holds a unique identifier for the user.
    exp: usize,  // `exp` is a timestamp representing the expiration time of the token.
}

// A public function that creates a JWT token for a given user ID.
// It takes a `Uuid` parameter representing the user's unique identifier and returns a String (the JWT).
pub fn create_token(user_id: Uuid) -> String {
    // Calculate expiration time for the token. This example sets it to one hour from the current time.
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(1))
        .unwrap() // Ensure the addition operation doesn't fail.
        .timestamp() as usize; // Convert the expiration time to a `usize`.

    // Create a `Claims` instance with the user ID as the subject and the calculated expiration time.
    let claims = Claims {
        sub: user_id.to_string(), // Convert `Uuid` to string to store in `sub`.
        exp: expiration,  // Set the expiration time.
    };

    // Encode the claims into a JWT using a default header and a secret key.
    // The secret key can be any byte array, but it should be kept secure and private.
    encode(
        &Header::default(), // Use default JWT header settings.
        &claims,            // Pass in the claims data.
        &EncodingKey::from_secret(b"secretkey"), // Secret key for encoding.
    )
    .unwrap() // Unwrap the result, assuming encoding is successful.
}

// A public function that validates a JWT token.
// It takes a string slice representing the token and returns a boolean indicating if the token is valid.
pub fn validate_token(token: &str) -> bool {
    // Attempt to decode the token using the same secret key used for encoding and default validation settings.
    decode::<Claims>(
        token,                                 // The JWT to be decoded.
        &DecodingKey::from_secret(b"secretkey"), // Secret key must match the one used during encoding.
        &Validation::default(),               // Use default validation parameters.
    )
    .is_ok() // Check if the decoding operation was successful.
}

// Hash password with salt
pub fn hash_with_salt(password: &str, salt: &str) -> Result<String, bcrypt::BcryptError> {
    let salted_password = format!("{}{}", salt, password);
    hash(salted_password, DEFAULT_COST)
}

// Verify password with salt
pub fn verify_with_salt(password: &str, salt: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    let salted_password = format!("{}{}", salt, password);
    verify(salted_password, hash)
}