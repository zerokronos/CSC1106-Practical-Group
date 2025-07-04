use actix_web::{Error, Result};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::env;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};

// For access control middleware
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    HttpMessage,
    http::header,
    error::ErrorUnauthorized,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req.headers().get(header::AUTHORIZATION);
            
            if let Some(header_value) = auth_header {
                if let Ok(auth_str) = header_value.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        
                        // Validate the token and extract user ID
                        if validate_token(token) {
                            if let Some(user_id) = extract_user_id_from_token(token) {
                                // Store user ID in request extensions for use in handlers
                                req.extensions_mut().insert(user_id);
                                
                                // Continue with the request
                                let fut = service.call(req);
                                return fut.await;
                            }
                        }
                    }
                }
            }
            
            // If we reach here, authentication failed
            Err(ErrorUnauthorized("Authentication required"))
        })
    }
}

// Helper function to extract user ID from request extensions
pub fn get_user_id_from_request(req: &ServiceRequest) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}

// Helper function for use in handlers
pub fn get_authenticated_user_id(req: &actix_web::HttpRequest) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}

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

// Function to extract user ID from a valid JWT token
pub fn extract_user_id_from_token(token: &str) -> Option<Uuid> {
    // Decode the token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(b"secretkey"),
        &Validation::default(),
    ).ok()?;

    // Parse the subject (user ID) from the claims
    Uuid::parse_str(&token_data.claims.sub).ok()
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