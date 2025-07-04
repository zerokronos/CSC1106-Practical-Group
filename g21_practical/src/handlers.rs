// Import necessary modules and types from external crates and internal modules.
// `actix_web` provides tools for building web servers, including handling HTTP requests and responses.
// `sqlx` is used for database connection pooling. `SqlitePool` is a specific pool type for SQLite.
// `uuid` is used for generating unique identifiers.
// `crate::models` and `crate::auth` denote relative imports from the current project's `models` and `auth` modules, respectively.
use actix_web::{web, HttpResponse, Responder, HttpRequest, Error, Result};
use sqlx::SqlitePool;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
// Models and authentication functionality needed for user, stock, and transaction handling.
use crate::models::{User, LoginRequest, LoginResponse, BugReport, CreateBug, Project};
use crate::auth;

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

// Define a function to configure the service, setting up the routes available in this web application.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login_function))) // POST /login
       .service(
    web::scope("")
                .wrap(AuthMiddleware)
                .service(web::resource("/projects").route(web::get().to(get_projects))) //GET /projects
                .service(web::resource("/projects").route(web::post().to(add_project))) //POST /projects
                .service(
                        web::scope("/bugs")
                            .route("", web::get().to(get_bugs)) // GET /bugs
                            .route("/{id}", web::get().to(get_bug_by_id)) // GET /bugs/{id}
                            .route("/new", web::post().to(create_bug)) // POST /bugs/new
                            .route("/assign", web::post().to(assign_bug)) // POST /bugs/assign
                            .route("/{id}", web::patch().to(update_bug_details)) // PATCH /bugs/{id}
                            .route("/{id}", web::delete().to(delete_bug)) //delete /bugs/{id}
       )
    );
}

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
                        if auth::validate_token(token) {
                            if let Some(user_id) = auth::extract_user_id_from_token(token) {
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

// Hash password with salt function
fn hash_with_salt(password: &str, salt: &str) -> Result<String, bcrypt::BcryptError> {
    let salted_password = format!("{}{}", salt, password);
    hash(salted_password, DEFAULT_COST)
}

// Function to verify password with salt
fn verify_with_salt(password: &str, salt: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    let salted_password = format!("{}{}", salt, password);
    verify(salted_password, hash)
}

// Asynchronous function for user login, expected to receive a JSON payload corresponding to a `User` object.
// Returns a responder, encapsulating an HTTP response.
async fn login_function(
    pool: web::Data<SqlitePool>, 
    body: web::Json<LoginRequest>
) -> impl Responder {
    let salt = "bugtrack2025";
    let user = sqlx::query_as::<_, User>(
             "SELECT id, username, hashed_password FROM users WHERE username = ?",
        )
        .bind(&body.username)
        .fetch_optional(pool.get_ref())
        .await;

    match user {
        Ok(Some(user)) => {
            // Verify password
            match verify_with_salt(&body.password, salt, &user.hashed_password) {
                Ok(true) => {
                    // Password correct, create token
                    let token = auth::create_token(user.id);

                    HttpResponse::Ok().json(LoginResponse {
                        status: "success".to_string(),
                        message: "Login successful".to_string(),
                        token: Some(token),
                    })
                }
                Ok(false) => {
                    // Password incorrect
                    HttpResponse::Unauthorized().json(LoginResponse {
                        status: "failure".to_string(),
                        message: "Invalid credentials".to_string(),
                        token: None,
                    })
                }
                Err(_) => {
                    // Error verifying password
                    HttpResponse::InternalServerError().json(LoginResponse {
                        status: "failure".to_string(),
                        message: "Internal server error".to_string(),
                        token: None,
                    })
                }
            }
        }
        Ok(None) => {
            // User not found
            HttpResponse::Unauthorized().json(LoginResponse {
                status: "failure".to_string(),
                message: "Invalid credentials".to_string(),
                token: None,
            })
        }
        Err(_) => {
            // Database error
            HttpResponse::InternalServerError().json(LoginResponse {
                status: "failure".to_string(),
                message: "Database error".to_string(),
                token: None,
            })
        }
    }
}

// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn get_projects(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn add_project(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn get_bugs(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn get_bug_by_id(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for creating a new bug report.
// Simply responds to the request with a confirmation message.
async fn create_bug(
    _pool: web::Data<SqlitePool>, 
    _body: web::Json<CreateBug>,
    _req: HttpRequest
) -> impl Responder {
    // Extract user ID from the request extensions
    let authenticated_user_id = match get_authenticated_user_id(&_req) {
        Some(user_id) => user_id,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "status": "error",
            "message": "Authentication required"
        }))
    };

    // Get the authenticated user from database
    let user = match sqlx::query_as::<_, User>(
        "SELECT id, username, hashed_password FROM users WHERE id = ?"
    )
    .bind(&authenticated_user_id)
    .fetch_optional(_pool.get_ref())
    .await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({
            "status": "error",
            "message": "User not found"
        })),
        Err(e) => {
            eprintln!("User query error: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Database error"
            }));
        }
    };

    // Get project by name
    let project = match sqlx::query_as::<_, ProjectRecord>(
        "SELECT id, project_name, project_description, created_at, user_id FROM projectRecord WHERE project_name = ?"
    )
    .bind(&_body.project_name)
    .fetch_optional(_pool.get_ref())
    .await {
        Ok(Some(project)) => project,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({
            "status": "error",
            "message": "Project not found"
        })),
        Err(e) => {
            eprintln!("Project query error: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Database error"
            }));
        }
    };

    let bug_id = uuid::Uuid::new_v4();
    
    // Insert bug report using authenticated user's ID
    if let Err(e) = sqlx::query("INSERT INTO bugReport (id, project_id, title, description, reported_by, severity, is_fixed) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&bug_id.as_bytes().as_slice())// Convert UUID to bytes for SQLite
        .bind(&project.id.as_bytes().as_slice())
        .bind(&_body.title)
        .bind(&_body.description)
        .bind(&user.id.as_bytes().as_slice())
        .bind(&_body.severity)
        .bind(false)
        .execute(_pool.get_ref())
        .await
    {
        eprintln!("BugReport insert error: {:?}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Failed to create bug report"
        }));
    }

    let response = BugReport {
        id: bug_id,
        project_id: project.id,
        title: _body.title.clone(),
        description: _body.description.clone(),
        reported_by: user.id,
        severity: _body.severity.clone(),
        fixed_by: uuid::Uuid::nil(), // Initially set to nil, as the bug is not fixed yet
        created_at: chrono::Utc::now().to_rfc3339(), // Current timestamp in RFC 3339 format
        is_fixed: false,
    };

    HttpResponse::Ok().json(response)
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn assign_bug(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn update_bug_details(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn delete_bug(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}